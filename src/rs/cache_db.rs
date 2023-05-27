use std::fmt::{Debug, Formatter};
use std::hash::Hash;
use std::{fmt, thread};
use std::ops::Deref;
use std::sync::{Arc, Condvar, Mutex, RwLock, MutexGuard, LockResult};
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};
use std::thread::{ThreadId};
use async_std::prelude::StreamExt;
use dashmap::DashMap;
use atomic_wait::{wait, wake_one, wake_all};
use mongodb::bson::{Bson, doc, Document, oid::ObjectId, to_document};
use mongodb::{bson, Client, Collection, Cursor};


use mongodb::options::{Acknowledgment, ClientOptions, DeleteOptions, InsertOneOptions, UpdateOptions, WriteConcern};
use mongodb::results::DeleteResult;
use schemars::_private::NoSerialize;
use serde::{Serialize, Deserialize, Serializer};
use serde::de::DeserializeOwned;
use serde::ser::SerializeStruct;
use crate::position_environment::Grade;


pub type DbKeyFn<K, T> = fn(&T) -> K;

#[derive(Deserialize, Debug, Clone)]
pub struct WrapItem<T>
    where
        T: Serialize + Unpin + Send + Sync + Clone,
{
    item: Arc<RwLock<T>>,
    #[serde(rename = "_id")]
    id: ObjectId,
    repetitions: u64,
    #[serde(skip)]
    write_counts: u16,
}

impl<T: Serialize> Serialize for WrapItem<T>
    where
        T: Serialize + Unpin + Send + Sync + Clone
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
    {
        let mut state = serializer.serialize_struct("WrapItem", 3)?;
        state.serialize_field("item", &*self.item.read().unwrap())?;
        state.serialize_field("repetitions", &self.repetitions)?;
        state.serialize_field("_id", &self.id)?;
        state.end()
    }
}

impl<T> From<WrapItem<T>> for Bson
    where
        T: Serialize + Unpin + Send + Sync + Clone + Debug,
{
    fn from(item: WrapItem<T>) -> Self {
        let json = bson::to_document(&item).unwrap();
        Bson::Document(json)
    }
}

impl<T> Into<Document> for WrapItem<T>
    where
        T: Serialize + Unpin + Send + Sync + Clone + Debug,
{
    fn into(self) -> Document {
        to_document(&self).unwrap()
    }
}

impl<T> WrapItem<T>
    where
        T: DeserializeOwned + Unpin + Send + Sync + Clone + Serialize,
{
    pub fn new(item: T) -> WrapItem<T> {
        WrapItem {
            item: Arc::new(RwLock::new(item)),
            id: ObjectId::default(),
            repetitions: 1,
            write_counts: 0,
        }
    }

    pub fn from_arc_item(item: Arc<RwLock<T>>) -> WrapItem<T> {
        WrapItem {
            item,
            id: ObjectId::default(),
            repetitions: 1,
            write_counts: 0,
        }
    }
    pub fn get_item(&self) -> Arc<RwLock<T>> {
        self.item.clone()
    }
    pub fn set_item(&mut self, item: Arc<RwLock<T>>) {
        self.item = item;
    }
}

pub enum WaitWhile {
    Del = 1,
    Rw = 2,
}

type Counter = Arc<(Mutex<u32>, Condvar)>;


pub struct CacheDb<K, T>
    where
        T: Serialize + DeserializeOwned + Unpin + Send + Sync + Clone,
        K: Hash + PartialEq + Serialize + 'static
{
    map: DashMap<K, WrapItem<T>>,
    locker: RwLock<bool>,
    key_fn: DbKeyFn<K, T>,
    db_name: String,
    collection_name: String,
    thread_dbc: DashMap<ThreadId, Collection<WrapItem<T>>>,
    size_limit: u64,
    item_update_every: u16,
    cut_collection_every: u16,
    inserts_count: Mutex<u64>,
}

impl<K, T> Debug for CacheDb<K, T>
    where
        T: Serialize + DeserializeOwned + Unpin + Send + Sync + Clone,
        K: Hash + Eq + Serialize + 'static,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("CacheDb")
            .field("db_name", &self.db_name)
            .field("collection_name", &self.collection_name)
            // Добавьте остальные поля, которые вам необходимо вывести
            .finish()
    }
}


impl<K, T> CacheDb<K, T>
    where
        T: Serialize + DeserializeOwned + Unpin + Send + Sync + Clone,
        K: Hash + Eq + Serialize + 'static {
    pub async fn new(key_fn: DbKeyFn<K, T>, db_name: String, collection_name: String, size_limit: u64, item_update_every: u16, cut_collection_every: u16) -> CacheDb<K, T> {
        CacheDb {
            map: DashMap::new(),
            thread_dbc: DashMap::new(),
            locker: RwLock::new(false),
            key_fn,
            db_name,
            collection_name,
            size_limit,
            item_update_every,
            cut_collection_every,
            inserts_count: Mutex::new(0),
        }
    }
    pub async fn init_database(&mut self) {
        let thread_id = thread::current().id();

        let client_options = ClientOptions::parse("mongodb://localhost:27017").await.unwrap();
        let client = Client::with_options(client_options).unwrap();
        let database = client.database(&self.db_name);
        let collection = database.collection::<WrapItem<T>>(&self.collection_name);
        self.thread_dbc.insert(thread_id, collection);
    }

    pub async fn drop_collection(&mut self) {
        let mut collection =
            self.thread_dbc.get_mut(&thread::current().id()).unwrap();
        collection.value_mut().drop(None).await.unwrap();
    }

    pub async fn read_collection(&mut self) {
        let collection = self.thread_dbc.get_mut(&thread::current().id()).unwrap();
        let mut cursor = collection.find(None, None).await.unwrap();

        let mut n = 0;
        while let Some(result) = cursor.next().await {
            match result {
                Ok(document) => {
                    n += 1;
                    let item = document.item.read().unwrap().clone();
                    let key = (self.key_fn)(&item);
                    let mut wrap_item = WrapItem::new(item);
                    wrap_item.id = document.id;
                    wrap_item.repetitions = document.repetitions;
                    self.map.insert(key, wrap_item);
                }
                Err(e) => {

                }
            }
        }
        println!("docs db: {} map: {}", n, self.map.len());
    }


    pub async fn collection_req(&mut self, pipeline: Vec<Document>) -> mongodb::error::Result<Cursor<Document>> {
        let mut collection =
            self.thread_dbc.get_mut(&thread::current().id()).unwrap();
        collection.value_mut().aggregate(pipeline, None).await
    }

    pub fn get(&self, key: &K) -> Option<Arc<RwLock<T>>> {
        let locker = self.locker.read();
        let item = self.map.get(key).as_ref().map(|entry| entry.value().item.clone()).clone();
        item
    }

    async fn flush(&self) {
        println!("{:?}", self.map.len());
        for x in self.map.iter() {
            if x.value().write_counts > 0 {
                self.db_update(&x).await;
            }
        }
    }

    async fn db_insert(&self, wrap_item: &mut WrapItem<T>) -> Bson {
        let collection =
            self.thread_dbc.get(&thread::current().id()).unwrap();

        let options = InsertOneOptions::builder()
            .write_concern(WriteConcern::builder().w(Acknowledgment::from(1)).build())
            .build();
        collection
            .insert_one(wrap_item, options).await.expect("db_insert error").inserted_id
    }

    async fn db_update(&self, wrap_item: &WrapItem<T>) {
        let collection =
            self.thread_dbc.get(&thread::current().id()).unwrap();
        let item_bson: Document = to_document(&wrap_item).unwrap();
        let filter = doc! { "_id": wrap_item.id };
        let update = doc! { "$set": item_bson };
        let options = UpdateOptions::builder().upsert(false).build();
        collection.update_one(filter, update, options).await.expect("db_update error");
    }

    async fn db_update_many(&self, wrap_items: &[WrapItem<T>]) {
        let collection =
            self.thread_dbc.get(&thread::current().id()).unwrap();
        let filter = doc! {"_id": {"$in": wrap_items.iter().map(|x| x.id).collect::<Vec<ObjectId>>()}};
        let update = doc! { "$set": wrap_items.iter().map(|x| to_document(&x).unwrap()).collect::<Vec<_>>() };
        let options = UpdateOptions::builder().upsert(false).build();
        collection.update_many(filter, update, options).await.expect("db_update error");
    }


    async fn db_cut(&self, cut_range: u32) -> mongodb::error::Result<DeleteResult> {
        let collection =
            self.thread_dbc.get(&thread::current().id()).unwrap();
        let filter = doc! { "repetitions": {"$lt": cut_range } };
        collection.delete_many(filter, DeleteOptions::default()).await
    }

    pub async fn insert(&self, item: T) {
        let lock = self.locker.read();
        let item = Arc::new(RwLock::new(item));
        let key = (self.key_fn)(&item.read().unwrap());
        let mut is_new = false;
        let mut val = self.map.entry(key).or_insert_with(|| {
            let mut count = self.inserts_count.lock().unwrap();
            *count += 1;
            is_new = true;
            WrapItem::from_arc_item(item.clone())
        });


        // update in db
        if !is_new {
            val.value_mut().repetitions += 1;
            val.value_mut().write_counts += 1;
            val.value_mut().set_item(item);
            if val.value().write_counts == self.item_update_every {
                val.value_mut().write_counts = 0;
                self.db_update(val.value()).await;
            }
        } else {
            // insert to db
            self.db_insert(&mut val).await.as_object_id().unwrap();
        }
        drop(val);
        drop(lock);

        let mut insert_count = self.inserts_count.lock().unwrap();
        if *insert_count >= self.cut_collection_every as u64 {
            let lock = self.locker.write();
            println!("start cutting..");
            let sum_rep: u64 =
                self.map.iter().map(|x| x.repetitions).sum();
            if sum_rep > 0 {
                let cut_range = sum_rep / self.map.len() as u64 / 70;
                let mut del = self.map.len();
                self.map.retain(|key, value| value.repetitions >= cut_range);
                del -= self.map.len();
                println!("flush..");
                self.flush().await;
                println!("cutting..");
                let res = self.db_cut(cut_range as u32).await;
                let del_from_db = if let Ok(res) = res {
                    res.deleted_count
                } else {
                    0
                };
                println!("db cut for: {:?} deleted from map: {}, db: {}", cut_range, del, del_from_db);
            }
            *insert_count = 0;
            drop(lock);
        }


        // println!("{:?}", self.map.len());


        // let cut_size: i64 = self.map.len() as i64 - self.size_limit as i64;
        //     if cut_size > 0 {
        //         self.inserts_count += 1;
        //         if self.inserts_count == self.cut_collection_every {
        //             self.inserts_count = 0;
        //             let mut sorted_entries: Vec<_> = self.map.iter().collect();
        //             sorted_entries.sort_by_key(|x| x.repetitions);
        //             for i in 0..cut_size as usize {
        //                 self.map.remove(sorted_entries[i].key());
        //                 // same in db
        //             }
        //         }
        //     }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, RwLock};
    use async_std::prelude::StreamExt;
    use mongodb::bson::{Bson, doc};
    use rand::Rng;
    use serde_derive::{Deserialize, Serialize};
    use tokio::time::Instant;
    use crate::cache_db::{CacheDb, DbKeyFn};
    use crate::mcts::OldCacheItem;

    #[derive(Debug, Deserialize, Serialize, Clone)]
    struct Test {
        v: Vec<i32>,
        n: i64,
    }

    impl Test {
        pub fn new(max_n: i64) -> Test {
            Test {
                v: (0..4).collect(),
                n: rand::thread_rng().gen_range(0..max_n),
            }
        }
    }


    #[tokio::test]
    async fn cache() {
        let key_fn: DbKeyFn<i64, Test> = |x| x.n;
        let db_name = String::from("test");
        let collection_name = String::from("items");
        let size_limit = 200;
        let item_update_every = 100;
        let cut_collection_every = 50;
        let max_n = 300;
        let iter_per_worker = 200000;
        let n_workers = 10;

        let cache_db = Arc::new(RwLock::new(
            CacheDb::new(
                key_fn, db_name, collection_name, size_limit,
                item_update_every, cut_collection_every).await),
        );

        cache_db.write().unwrap().init_database().await;
        cache_db.write().unwrap().read_collection().await;
        let mut repetitions = get_repetitions(&cache_db).await;

        let time = Instant::now();
        let mut xx = vec![];
        for _ in 0..n_workers {
            let db = cache_db.clone();

            let x = tokio::task::spawn_blocking(move || {
                tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .unwrap()
                    .block_on(async {
                        db.write().unwrap().init_database().await;
                        for _ in 0..iter_per_worker {
                            db.read().unwrap().insert(Test::new(max_n)).await;
                        }
                    });
            });
            xx.push(x);
        }
        for x in xx {
            let y = x.await;
        }

        async fn get_repetitions(cache_db: &Arc<RwLock<CacheDb<i64, Test>>>) -> i64 {
            let pipeline = vec![
                doc! {
            "$group": {
                "_id": null,
                "totalRepetitions": { "$sum": "$repetitions" }
            }
        }
            ];
            let doc =
                cache_db.write().unwrap().collection_req(pipeline).await.unwrap().next().await;
            doc.unwrap().unwrap().get("totalRepetitions").and_then(Bson::as_i64).unwrap()
        }
        cache_db.write().unwrap().flush().await;


        println!("{:?}", time.elapsed());

        let pipeline = vec![
            doc! {
            "$group": {
                "_id": null,
                "totalRepetitions": { "$sum": "$repetitions" }
            }
        }
        ];
        let doc =
            cache_db.write().unwrap().collection_req(pipeline).await.unwrap().next().await;
        repetitions = get_repetitions(&cache_db).await - repetitions;
        assert_eq!(repetitions, iter_per_worker * n_workers);

        let pipeline = vec![
            doc! {
            "$count": "totalItems"
        }];
        let mut doc =
            cache_db.write().unwrap().collection_req(pipeline).await.unwrap().next().await;
        let x = doc.as_mut().unwrap().as_mut().unwrap().get("totalItems").unwrap();
        let total_items = x.clone().as_i32().unwrap();
        assert_eq!(total_items as i64, max_n);
    }
}
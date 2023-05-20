use std::fmt::Debug;
use std::hash::Hash;
use std::io;
use std::io::Write;
use std::sync::{Arc, Mutex, RwLock};
use dashmap::DashMap;
use mongodb::bson::{Bson, doc, Document, oid::ObjectId, to_document};
use mongodb::{bson, Client, Database};

use mongodb::options::{ClientOptions, FindOneOptions, UpdateOptions};
use serde::{Serialize, Deserialize, Serializer};
use serde::de::DeserializeOwned;
use serde::ser::SerializeStruct;


pub type DbKeyFn<K, T> = fn(&T) -> K;

#[derive(Deserialize, Debug)]
pub struct WrapItem<T>
    where
        T: Serialize + Unpin + Send + Sync + Clone + Debug,
{
    item: RwLock<T>,
    #[serde(rename = "_id")]
    id: ObjectId,
    repetitions: u64,
    #[serde(skip)]
    write_counts: u16,
}

impl<T: Serialize> Serialize for WrapItem<T>
    where
        T: Serialize + Unpin + Send + Sync + Clone + Debug
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
    {
        let mut state = serializer.serialize_struct("WrapItem", 2)?;
        state.serialize_field("item", &*self.item.read().unwrap())?;
        state.serialize_field("repetitions", &self.repetitions)?;
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
        T: DeserializeOwned + Unpin + Send + Sync + Clone + Serialize + Debug,
{
    pub fn new(item: T) -> WrapItem<T> {
        WrapItem {
            item: RwLock::new(item),
            id: ObjectId::default(),
            repetitions: 0,
            write_counts: 0,
        }
    }
}

pub struct CacheDb<K, T>
    where
        T: Serialize + DeserializeOwned + Unpin + Send + Sync + Clone + Debug,
        K: Hash + Eq + Serialize + 'static + Clone + Debug
{
    map: DashMap<K, WrapItem<T>>,
    key_fn: DbKeyFn<K, T>,
    db: Database,
    size_limit: u64,
    item_update_every: u16,
    cut_collection_every: u16,
    inserts_count: u16,
}

impl<K, T> CacheDb<K, T>
    where
        T: Serialize + DeserializeOwned + Unpin + Send + Sync + Clone + Debug,
        K: Hash + Eq + Serialize + 'static + Clone + Debug {
    pub async fn new(key_fn: DbKeyFn<K, T>, db_name: String, size_limit: u64, item_update_every: u16, cut_collection_every: u16) -> CacheDb<K, T> {
        CacheDb {
            map: DashMap::new(),
            db: {
                let client_options = ClientOptions::parse("mongodb://localhost:27017").await.unwrap();
                let client = Client::with_options(client_options).unwrap();
                client.database(&db_name)
            },
            key_fn,
            size_limit,
            item_update_every,
            cut_collection_every,
            inserts_count: 0,
        }
    }

    pub async fn insert(&mut self, item: T) {
        let key = (self.key_fn)(&item);
        self.map.entry(key.clone()).or_insert_with(|| {
            println!("{:?}", key.clone());
            WrapItem::new(item)
        });

        if let Some(mut value) = self.map.get_mut(&key) {
            value.value_mut().repetitions += 1;
            value.value_mut().write_counts += 1;
            if value.write_counts == self.item_update_every {
                value.value_mut().write_counts = 0;
                // update in db
                let collection = self.db.collection::<WrapItem<T>>("items");
                let filter = doc! { "_id": value.id };
                println!("{:?}", filter);
                io::stdout().flush().unwrap();
                let item_bson: Document = to_document(&value.value()).unwrap();
                let result = collection.find_one(filter, FindOneOptions::builder().build()).await;
                if let Ok(res) = result {
                    if let Some(item) = res {
                        let filter = doc! { "_id": value.id };
                        let update = doc! { "$set": item_bson };
                        let options = UpdateOptions::builder().upsert(false).build();
                        collection.update_one(filter, update, options).await.unwrap();
                    }
                    else {
                        let res = collection
                            .insert_one(value.value(), None).await.unwrap();
                        let id = res.inserted_id.as_object_id().unwrap();
                        println!("{:?} {:?}", id, value.value());
                        value.value_mut().id = id;
                    }
                }
            }
        }


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
    use crate::cache_db::{CacheDb, DbKeyFn};

    #[tokio::test]
    async fn cache() {
        let key_fn: DbKeyFn<i32, i32> = |x| *x;
        let db_name = String::from("test");
        let size_limit = 100;
        let item_update_every = 1;
        let cut_collection_every = 20;

        let mut cache_db =
            CacheDb::new(key_fn, db_name, size_limit, item_update_every, cut_collection_every).await;

        // Use the `cache_db` instance...
        cache_db.insert(42).await;
        cache_db.insert(42).await;
        cache_db.insert(43).await;
    }
}

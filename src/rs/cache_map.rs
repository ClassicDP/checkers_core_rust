use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::hash::Hash;
use std::io::{BufWriter, Read, Write};
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};
use serde_json::to_writer;
use wasm_bindgen::prelude::wasm_bindgen;
use bincode::serialize_into;
use serde_with::rust::maps_first_key_wins::serialize;
use crate::loop_array::LoopArray;
use crate::position::{Position, PositionKey};


trait Repetition {
    fn n(&self) -> i32;
    fn repeat(&mut self);
    fn get_array_pointer(&self) -> usize;
    fn set_list_pointer(&mut self, pointer: usize);
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Wrapper<T> {
    pub item: T,
    pub repetitions: i32,
    array_pointer: usize,
}

impl<T> Wrapper<T> {
    pub fn new(item: T) -> Wrapper<T> {
        Wrapper {
            item,
            array_pointer: 0,
            repetitions: 0,
        }
    }
}

impl<T> Repetition for Wrapper<T> {
    fn n(&self) -> i32 {
        self.repetitions
    }

    fn repeat(&mut self) {
        self.repetitions += 1;
    }

    fn get_array_pointer(&self) -> usize {
        self.array_pointer
    }

    fn set_list_pointer(&mut self, pointer: usize) {
        self.array_pointer = pointer;
    }
}

#[derive(Serialize, Deserialize)]
pub struct CacheMap<K, T>
    where
        T: Serialize,
        K: Hash + Eq + Serialize + 'static

{
    pub freq_list: LoopArray<Arc<Mutex<Wrapper<T>>>>,
    #[serde(skip_serializing)]
    map: HashMap<K, Arc<Mutex<Wrapper<T>>>>,
    size: usize,
    max_size: usize,
    #[serde(skip)]
    key_fn: Option<KeyFn<T, K>>,
}

impl<K, T> Debug for CacheMap<K, T>
    where
        T: Serialize + Debug,
        K: Hash + Eq + Serialize + 'static
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &format!("{:?}", self.freq_list))
    }
}

pub type KeyFn<T, K> = fn(&T) -> K;

impl<K, T> CacheMap<K, T>
    where
        T: Serialize + for<'d> Deserialize<'d> + Clone,
        K: Hash + Eq + Serialize
{
    pub fn new(key_fn: KeyFn<T, K>, max_size: usize) -> CacheMap<K, T> {
        let mut v: LoopArray<Arc<Mutex<Wrapper<T>>>> = LoopArray::new(max_size);
        v.v.resize_with(max_size, || None);
        CacheMap {
            map: HashMap::new(),
            max_size,
            size: 0,
            freq_list: v,
            key_fn: Some(key_fn),
        }
    }

    pub fn set_repetitions(&mut self, x: &T, repetitions: i32) {
        let key = self.key(x);
        let z = self.map.get(&key);
        if z.is_some() { z.unwrap().lock().unwrap().repetitions = repetitions };
    }

    pub fn insert(&mut self, x: T) {
        let key = &self.key(&x);
        let v = self.map.get(key);
        if let Some(v) = v {
            v.lock().unwrap().repeat();
            v.lock().unwrap().item = x;
            let i = v.lock().unwrap().get_array_pointer();
            let i_next = self.freq_list.next_loop_p(i);
            if i_next.is_some() {
                let i_next = i_next.unwrap();
                self.freq_list.swap(i, i_next);
                self.freq_list.v[i].as_ref().unwrap().lock().unwrap().array_pointer = i;
                self.freq_list.v[i_next].as_ref().unwrap().lock().unwrap().array_pointer = i_next;
            }
        } else {
            let wrap = Arc::from(Mutex::from(Wrapper::new(x)));
            let (old, pointer) = self.freq_list.push(wrap.clone());
            if old.is_some() {
                let key = self.key(&old.as_ref().unwrap().as_ref().lock().unwrap().item);
                self.map.remove(&key);
            }
            wrap.lock().unwrap().set_list_pointer(pointer);
            let key = self.key(&wrap.lock().unwrap().item);
            self.map.insert(key, wrap.clone());
        }
    }

    pub fn key(&mut self, item: &T) -> K {
        (self.key_fn.unwrap())(item)
    }

    pub fn get(&mut self, key: &K) -> Option<Arc<Mutex<Wrapper<T>>>> {
        let x = self.map.get(key);
        if x.is_some() { Some(x.unwrap().clone()) } else { None }
    }

    pub fn resort<SK: Ord>(&mut self, sort_fn: fn(item: Option<T>) -> SK) {
        self.freq_list.v.sort_by_key(|x| (sort_fn)(Some(
            x.as_ref().unwrap().lock().unwrap().item.clone())));
        let list = self.freq_list.v.clone();
        let mut cache_map = CacheMap::new(self.key_fn.unwrap(), self.max_size);
        self.map = HashMap::new();
        for x in list {
            if x.is_some() { cache_map.insert(x.unwrap().lock().unwrap().item.clone()); }
        }
        self.freq_list = cache_map.freq_list;
        self.map = cache_map.map;
        self.size = cache_map.size;
        self.max_size = cache_map.max_size;
    }

    pub fn get_cache_json(&self) -> String {
        serde_json::to_string(&self.freq_list).unwrap()
    }

    pub fn write(&mut self, f_name: String) {
        fn buck_up(f_name: &String) -> std::io::Result<()> {
            let file_path = f_name.clone();
            let backup_file_path = f_name.clone() + ".bak";

            // Open the file for reading
            let file = std::fs::File::open(&file_path);
            if file.is_err() { return Ok(()); }
            // Read the contents of the file
            let mut contents = String::new();
            file.unwrap().read_to_string(&mut contents)?;

            // Create a backup file and write the contents to it
            let mut backup_file = std::fs::File::create(&backup_file_path)?;
            backup_file.write_all(contents.as_bytes())?;
            Ok(())
        }
        if buck_up(&f_name).is_err() {
            panic!("cant backup cache file");
        }
        let s = self.get_cache_json();
        let mut f = BufWriter::new(std::fs::File::create(f_name).unwrap());
        f.write(s.as_ref()).unwrap();
        f.flush().unwrap();
    }

    pub fn from_file(f_name: String, key_fn: KeyFn<T, K>, max_size: usize) -> CacheMap<K, T> {
        let read_freq_list = || {
            let file = std::fs::File::open(f_name);
            match file {
                Ok(mut file) => {
                    let mut contents = String::new();
                    file.read_to_string(&mut contents)?;
                    let mut list: LoopArray<Arc<Mutex<Wrapper<T>>>> = serde_json::from_str(&contents)?;
                    Ok(list)
                }
                Err(e) => {
                    print!("{}\n", e);
                    std::io::stdout().flush().unwrap();
                    Err(e)
                }
            }
        };
        let freq_list: Result<LoopArray<Arc<Mutex<Wrapper<T>>>>, std::io::Error> = read_freq_list();
        match freq_list {
            Ok(freq_list) => {
                let mut new_map = CacheMap::new(key_fn, max_size);
                let list = freq_list.get_list();
                for x in list {
                    new_map.insert(x.lock().unwrap().item.clone());
                    let item = x.lock().unwrap().item.clone();
                    new_map.set_repetitions(&item, x.lock().unwrap().repetitions);
                }
                new_map
            }
            Err(e) => {
                print!("{}\n", e);
                std::io::stdout().flush().unwrap();
                CacheMap::new(key_fn, max_size)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::fmt::{Debug, Formatter};
    use std::hash::{Hash, Hasher};
    use std::rc::Rc;
    use serde_derive::Serialize;
    use crate::cache_map::{CacheMap, Repetition};


    #[test]
    fn map_test() {
        #[derive(Debug, Serialize)]
        struct VecKeyStruct {
            n: i32,
            v: Vec<i32>,
            list_index: usize,
        }
        impl Repetition for VecKeyStruct {
            fn n(&self) -> i32 {
                self.n
            }

            fn repeat(&mut self) {
                self.n += 1;
            }

            fn get_array_pointer(&self) -> usize {
                self.list_index
            }

            fn set_list_pointer(&mut self, pointer: usize) {
                self.list_index = pointer;
            }
        }
        impl VecKeyStruct {
            fn new(v: Vec<i32>) -> VecKeyStruct {
                VecKeyStruct {
                    v,
                    n: 0,
                    list_index: 0,
                }
            }
        }

        let a1 = vec![1, 1, 1, 1];
        let a2 = vec![2, 2, 2, 2];
        let a3 = vec![3, 3, 3, 3];
        let a4 = vec![4, 4, 4, 4];
        let mut cache_map = CacheMap::new(|x: &Vec<i32>| x.clone(), 3);
        cache_map.insert(a1.clone());
        cache_map.insert(a2.clone());
        cache_map.insert(a3.clone());
        cache_map.insert(a2.clone());
        print!("{:?}\n", cache_map.freq_list.get_list());
        assert_eq!(cache_map.freq_list.at(2).as_ref().unwrap().as_ref().lock().unwrap().item, a2);
        cache_map.insert(a1.clone());
        assert_eq!(cache_map.freq_list.at(1).as_ref().unwrap().as_ref().lock().unwrap().item, a1);
        print!("{:?}\n", cache_map.freq_list.get_list());
        cache_map.insert(a2.clone());
        assert_eq!(cache_map.freq_list.at(2).as_ref().unwrap().as_ref().lock().unwrap().item, a2);
        print!("{:?}\n", cache_map.freq_list.get_list());
        cache_map.write("cache.json".to_string());
        let mut new_cache = CacheMap::from_file("cache.json".to_string(), |x: &Vec<i32>| x.clone(), 3);
        print!("{:?}\n", new_cache.freq_list.get_list());
        new_cache.insert(a1.clone());
        print!("{:?}\n", new_cache.freq_list.get_list());
        new_cache.insert(a2);
        print!("{:?}\n", new_cache.freq_list.get_list());
        new_cache.insert(a4.clone());
        print!("{:?}\n", new_cache.freq_list.get_list());
        assert_eq!(new_cache.freq_list.at(0).as_ref().unwrap().lock().unwrap().item, a1);
        assert_eq!(new_cache.freq_list.at(2).as_ref().unwrap().lock().unwrap().item, a4);
    }
}
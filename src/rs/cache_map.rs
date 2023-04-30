use std::cell::RefCell;
use std::collections::HashMap;
use std::hash::Hash;
use std::io::{Read, Write};
use std::rc::Rc;
use serde::{Deserialize, Serialize};
use serde_json::to_writer;
use serde_with::serde_as;

trait Repetition {
    fn n(&self) -> i32;
    fn repeat(&mut self);
    fn get_list_ind(&self) -> usize;
    fn set_list_ind(&mut self, ind: usize);
}

#[derive(Debug, Serialize, Deserialize)]
struct Wrapper<T> {
    item: T,
    repetitions: i32,
    index: usize,
}

impl<T> Wrapper<T> {
    pub fn new(item: T) -> Wrapper<T> {
        Wrapper {
            item,
            index: 0,
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

    fn get_list_ind(&self) -> usize {
        self.index
    }

    fn set_list_ind(&mut self, ind: usize) {
        self.index = ind;
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde_as]
struct CacheMap<K, FK, T>
    where
        FK: Fn(&T) -> K,
        T: Serialize,
        K: Hash + Eq + Serialize

{
    freq_list: Vec<Option<Rc<RefCell<Wrapper<T>>>>>,
    #[serde(skip_serializing)]
    map: HashMap<K, Rc<RefCell<Wrapper<T>>>>,
    size: usize,
    max_size: usize,
    #[serde(skip_serializing)]
    key_fn: FK,
}

impl<K, FK, T> CacheMap<K, FK, T>
    where
        T: Serialize + for <'d> Deserialize<'d>,
        FK: Fn(&T) -> K,
        K: Hash + Eq + Serialize
{
    fn new(key_fn: FK, max_size: usize) -> CacheMap<K, FK, T> {
        let mut v: Vec<Option<Rc<RefCell<Wrapper<T>>>>> = vec![];
        v.resize_with(max_size, || None);
        CacheMap {
            map: HashMap::new(),
            max_size,
            size: 0,
            freq_list: v,
            key_fn,
        }
    }

    pub fn insert(&mut self, x: T) {
        let v = self.map.get(&(self.key_fn)(&x));
        if let Some(mut v) = v {
            v.borrow_mut().repeat();
            let i = v.borrow_mut().get_list_ind();
            if i > 0 && self.freq_list[i - 1].as_ref().unwrap().borrow_mut().n() <
                self.freq_list[i].as_ref().unwrap().borrow_mut().n() {
                self.freq_list.swap(i, i - 1);
                self.freq_list[i - 1].as_ref().unwrap().borrow_mut().set_list_ind(i - 1);
                self.freq_list[i].as_ref().unwrap().borrow_mut().set_list_ind(i);
            }
        } else {
            let wrap = Rc::from(RefCell::from(Wrapper::new(x)));
            if self.size < self.max_size {
                self.freq_list[self.size] = Option::from(wrap.clone());
                wrap.borrow_mut().set_list_ind(self.size);
                self.size += 1;
                self.map.insert((self.key_fn)(&wrap.borrow().item), wrap.clone());
            } else {
                self.map.remove(&(self.key_fn)(&self.freq_list[self.size - 1].as_ref().unwrap().borrow().item));
                self.freq_list[self.size - 1] = Option::from(wrap.clone());
                wrap.borrow_mut().set_list_ind(self.size - 1);
                self.map.insert((self.key_fn)(&wrap.borrow().item), wrap.clone());
            }
        }
    }

    fn write(&self, f_name: String) {
        let mut file = std::fs::File::create(f_name).expect("Open file error");
        to_writer(&file, &self.freq_list).expect("Write file error");
        file.flush().expect("Write file error flush");
    }

    fn from_file(f_name: String, key_fn: FK, max_size: usize) -> CacheMap<K, FK, T> {
        let mut file = std::fs::File::open(f_name).expect("Open file error");
        let mut contents = String::new();
        file.read_to_string(&mut contents).expect("Read file error");
        let mut freq_list: Vec<Option<Rc<RefCell<Wrapper<T>>>>> = serde_json::from_str(&contents).expect("Format data error");
        let mut map = HashMap::new();
        for item in &freq_list {
            map.insert((key_fn)(&item.as_ref().unwrap().borrow().item), item.as_ref().unwrap().clone());
        }
        let size = freq_list.len();
        freq_list.resize_with(max_size, || None);
        CacheMap {
            max_size,
            key_fn,
            freq_list,
            map,
            size
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

            fn get_list_ind(&self) -> usize {
                self.list_index
            }

            fn set_list_ind(&mut self, ind: usize) {
                self.list_index = ind;
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
        let mut cache_map = CacheMap::new(|x: &Vec<i32>| x.clone(), 2);
        cache_map.insert(a1);
        cache_map.insert(a2.clone());
        print!("{:?}\n", cache_map.freq_list);
        cache_map.insert(a2);
        print!("{:?}\n", cache_map.freq_list);
        cache_map.insert(a3.clone());
        print!("{:?}\n", cache_map.freq_list);
        cache_map.write("cache.json".to_string());
        let mut new_cache = CacheMap::from_file("cache.json".to_string(), |x: &Vec<i32>| x.clone(), 20);
        new_cache.insert(a3.clone());
        new_cache.insert(a3);
        print!("{:?}\n", new_cache.freq_list);

    }
}
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::hash::Hash;
use std::io::{Read, Write};
use std::rc::Rc;
use serde::{Deserialize, Serialize};
use serde_json::to_writer;
use serde_with::serde_as;
use wasm_bindgen::prelude::wasm_bindgen;
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
    item: T,
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
#[serde_as]
pub struct CacheMap<K, T>
    where
        T: Serialize,
        K: Hash + Eq + Serialize + 'static

{
    pub freq_list: LoopArray<Rc<RefCell<Wrapper<T>>>>,
    #[serde(skip_serializing)]
    map: HashMap<K, Rc<RefCell<Wrapper<T>>>>,
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
        T: Serialize + for<'d> Deserialize<'d>,
        K: Hash + Eq + Serialize
{
    pub fn new(key_fn: KeyFn<T, K>, max_size: usize) -> CacheMap<K, T> {
        let mut v: LoopArray<Rc<RefCell<Wrapper<T>>>> = LoopArray::new(max_size);
        v.v.resize_with(max_size, || None);
        CacheMap {
            map: HashMap::new(),
            max_size,
            size: 0,
            freq_list: v,
            key_fn: Some(key_fn),
        }
    }

    pub fn insert(&mut self, x: T) {
        let key = &(self.key_fn.unwrap())(&x);
        let v = self.map.get(key);
        if let Some(v) = v {
            v.borrow_mut().repeat();
            v.borrow_mut().item = x;
            let i = v.borrow_mut().get_array_pointer();
            let i_next = self.freq_list.next_loop_p(i);
            if i_next.is_some() {
                let i_next = i_next.unwrap();
                self.freq_list.swap(i, i_next);
                self.freq_list.v[i].as_ref().unwrap().borrow_mut().array_pointer = i;
                self.freq_list.v[i_next].as_ref().unwrap().borrow_mut().array_pointer = i_next;
            }
        } else {
            let wrap = Rc::from(RefCell::from(Wrapper::new(x)));
            let (old, pointer) = self.freq_list.push(wrap.clone());
            if old.is_some() {
                self.map.remove(
                    &(self.key_fn.unwrap())(&old.as_ref().unwrap().as_ref().borrow().item));
            }
            wrap.borrow_mut().set_list_pointer(pointer);
            self.map.insert((self.key_fn.unwrap())(&wrap.borrow().item), wrap.clone());
        }
    }


    fn write(&self, f_name: String) {
        let mut file = std::fs::File::create(f_name).expect("Open file error");
        to_writer(&file, &self.freq_list).expect("Write file error");
        file.flush().expect("Write file error flush");
    }

    fn from_file(f_name: String, key_fn: KeyFn<T, K>, max_size: usize) -> CacheMap<K, T> {
        let mut file = std::fs::File::open(f_name).expect("Open file error");
        let mut contents = String::new();
        file.read_to_string(&mut contents).expect("Read file error");
        let mut freq_list: LoopArray<Rc<RefCell<Wrapper<T>>>> = serde_json::from_str(&contents).expect("Format data error");
        let mut map = HashMap::new();
        for item in &freq_list.v {
            map.insert((key_fn)(&item.as_ref().unwrap().borrow().item), item.as_ref().unwrap().clone());
        }
        let size = freq_list.v.len();
        freq_list.v.resize_with(max_size, || None);
        CacheMap {
            max_size,
            key_fn: Some(key_fn),
            freq_list,
            map,
            size,
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
        assert_eq!(cache_map.freq_list.at(2).as_ref().unwrap().as_ref().borrow_mut().item, a2);
        cache_map.insert(a1.clone());
        assert_eq!(cache_map.freq_list.at(1).as_ref().unwrap().as_ref().borrow_mut().item, a1);
        print!("{:?}\n", cache_map.freq_list.get_list());
        cache_map.insert(a2.clone());
        assert_eq!(cache_map.freq_list.at(2).as_ref().unwrap().as_ref().borrow_mut().item, a2);
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
        assert_eq!(new_cache.freq_list.at(0).as_ref().unwrap().borrow_mut().item, a1);
        assert_eq!(new_cache.freq_list.at(2).as_ref().unwrap().borrow_mut().item, a4);
    }
}
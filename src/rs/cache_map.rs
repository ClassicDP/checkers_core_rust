use std::cell::RefCell;
use std::collections::HashMap;
use std::hash::Hash;
use std::rc::Rc;

trait Repetition {
    fn n(&self) -> i32;
    fn repeat(&mut self);
    fn get_list_ind(&self) -> usize;
    fn set_list_ind(&mut self, ind: usize);
}

#[derive(Debug)]
struct CacheMap<K, FK, T>
    where
        FK: Fn(&Rc<RefCell<T>>) -> K,
        K: Hash + Eq
{
    freq_list: Vec<Option<Rc<RefCell<T>>>>,
    map: HashMap<K, Rc<RefCell<T>>>,
    key_fn: FK,
    size: usize,
    max_size: usize,
}

impl<K, FK, T: Repetition> CacheMap<K, FK, T>
    where
        FK: Fn(&Rc<RefCell<T>>) -> K,
        K: Hash + Eq
{
    fn new(key_fn: FK, max_size: usize) -> CacheMap<K, FK, T> {
        let mut v: Vec<Option<Rc<RefCell<T>>>> = vec![];
        v.resize_with(max_size, || None);
        CacheMap {
            map: HashMap::new(),
            max_size,
            size: 0,
            freq_list: v,
            key_fn,
        }
    }
    pub fn insert(&mut self, x: Rc<RefCell<T>>) {
        let v = self.map.get(&(self.key_fn)(&x));
        if let Some(v) = v {
            v.borrow_mut().repeat();
            let i = v.borrow_mut().get_list_ind();
            if i > 0 && self.freq_list[i - 1].as_ref().unwrap().borrow().n() <
                self.freq_list[i].as_ref().unwrap().borrow().n() {
                self.freq_list.swap(i, i - 1);
                self.freq_list[i - 1].as_ref().unwrap().borrow_mut().set_list_ind(i - 1);
                self.freq_list[i].as_ref().unwrap().borrow_mut().set_list_ind(i);
            }
        } else {
            if self.size < self.max_size {
                self.freq_list[self.size] = Option::from(x.clone());
                x.borrow_mut().set_list_ind(self.size);
                self.size += 1;
                self.map.insert((self.key_fn)(&x), x.clone());
            } else {
                self.map.remove(&(self.key_fn)(&self.freq_list[self.size - 1].clone().unwrap()));
                self.freq_list[self.size - 1] = Option::from(x.clone());
                self.map.insert((self.key_fn)(&x), x.clone());
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
    use crate::cache_map::{CacheMap, Repetition};


    #[test]
    fn map_test() {
        #[derive(Debug)]
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
        let a1 = vec![0, 1, 2, 3];
        let a2 = vec![0, 1, 3, 3];
        let vk1 = Rc::new(RefCell::new(VecKeyStruct::new(a1)));
        let mut vk2 = Rc::new(RefCell::new(VecKeyStruct::new(a2)));
        let a3 = vec![0, 3, 3, 3];
        let vk3 = Rc::new(RefCell::new(VecKeyStruct::new(a3)));
        let mut cash_map = CacheMap::new(|x: &Rc<RefCell<VecKeyStruct>>| x.borrow().v.clone(), 2);
        cash_map.insert(vk1.clone());
        cash_map.insert(vk2.clone());
        print!("{:?}\n", cash_map.freq_list);
        cash_map.insert(vk2.clone());
        print!("{:?}\n", cash_map.freq_list);
        cash_map.insert(vk3);
        print!("{:?}\n", cash_map.freq_list);
    }
}
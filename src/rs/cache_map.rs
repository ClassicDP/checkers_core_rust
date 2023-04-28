use std::cell::RefCell;
use std::collections::HashMap;
use std::hash::Hash;
use std::rc::Rc;

trait Repetition {
    fn n(&self) -> i32;
    fn get_list_ind(&self) -> usize;
    fn set_list_ind(&self, ind: usize);
}

struct CacheMap<K, FK, T>
    where
        FK: Fn(&Rc<RefCell<T>>) -> K,
        K: Hash + Eq
{
    freq_list: Vec<Rc<RefCell<T>>>,
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
        CacheMap {
            map: HashMap::new(),
            max_size,
            size: 0,
            freq_list: Vec::with_capacity(max_size),
            key_fn,
        }
    }
    pub fn insert(&mut self, x: Rc<RefCell<T>>) {
        let v = self.map.get(&(self.key_fn)(&x));
        if let Some(v) = v {
            let i = v.borrow_mut().get_list_ind();
            if i > 0 && self.freq_list[i - 1].borrow_mut().n() < self.freq_list[i].borrow_mut().n() {
                self.freq_list.swap(i, i - 1);
                self.freq_list[i - 1].borrow_mut().set_list_ind(i - 1);
                self.freq_list[i].borrow_mut().set_list_ind(i);
            }
        } else {
            if self.size < self.max_size {
                self.freq_list[self.size] = x.clone();
                x.borrow_mut().set_list_ind(self.size);
                self.size += 1;
                self.map.insert((self.key_fn)(&x), x.clone());
            } else {
                self.map.remove(&(self.key_fn)(&self.freq_list[self.size-1]));
                self.freq_list[self.size-1] = x.clone();
                self.map.insert((self.key_fn)(&x), x.clone());
            }
        }
    }
}
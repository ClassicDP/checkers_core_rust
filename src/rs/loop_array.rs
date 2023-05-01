use serde::Serialize;
use serde_derive::Deserialize;

#[derive(Debug, Serialize, Deserialize)]
pub struct LoopArray<T>
    where
        T: Serialize,
{
    pub v: Vec<T>,
    pub data_size: usize,
    pub max_size: usize,
    p: usize,
}

impl<T> LoopArray<T>
    where
        T: Serialize,
{
    pub fn new(max_size: usize) -> LoopArray<T> {
        LoopArray {
            v: vec![],
            data_size: 0,
            max_size,
            p: 0,
        }
    }

    fn next_p(&mut self) {
        self.p = (self.p + 1) % self.max_size;
    }

    fn prev_p(&mut self) {
        self.p = (self.p + self.data_size - 1) % self.data_size;
    }

    pub fn next_loop_p(&mut self) -> Option<usize> {
        let p = (self.p + 1) % self.data_size;
        if p == self.p { None } else { Some(p) }
    }

    fn prev_loop_p(&mut self) -> usize {
        (self.p + self.data_size - 1) % self.data_size
    }

    fn get_p(&self) -> usize {
        self.p
    }

    fn get_data_size(&self) -> usize {
        self.data_size
    }

    pub fn swap(&mut self, p1: usize, p2: usize) {
        self.v.swap(p1, p2);
    }

    fn i(&self, i: usize) -> usize {
        (i + self.p + self.max_size - self.data_size) % self.max_size
    }

    fn get_index(&self, pointer: usize) -> usize {
        (pointer + self.data_size + self.max_size - self.p) % self.max_size
    }

    pub fn at(&self, i: usize) -> &T {
        &self.v[self.i(i)]
    }

    pub fn set(&mut self, i: usize, val: T) {
        if i < self.data_size {
            let ind = self.i(i);
            self.v[ind] = val;
        }
    }

    pub fn push(&mut self, val: T) -> usize {
        self.v[self.p] = val;
        let pointer = self.p;
        if self.data_size < self.max_size { self.data_size += 1; }
        self.next_p();
        pointer
    }
}
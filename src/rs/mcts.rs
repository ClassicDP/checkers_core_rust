use std::cell::{Cell, RefCell};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::rc::Rc;
use crate::position::{Position, TuplePositionKey};
use crate::PositionHistory::{FinishType, PositionAndMove, PositionHistory};
use rand::{Rng};
use serde::Serialize;
use crate::color::Color;
use crate::color::Color::{Black, White};
use crate::moves_list::MoveItem;
use crate::piece::Piece;
use serde::Deserialize;
use std::iter::Iterator;
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use dashmap::DashMap;
use js_sys::Map;
use crate::cache_db::CacheDb;

use ts_rs::*;

#[derive(Serialize, PartialEq, Eq, Debug, Clone, Deserialize, Hash, TS)]
pub struct VectorPosition(Arc<Vec<i8>>);

#[derive(Serialize, Debug, Clone, Deserialize)]
pub struct PositionQuality {
    pub W: i64,
    pub N: i64,
    pub NN: i64,
}


impl VectorPosition {
    pub fn from_position(pos: &Position) -> VectorPosition {
        VectorPosition::from_cells(&pos.cells, pos.next_move.unwrap())
    }

    pub fn from_cells(cells: &Vec<Option<Piece>>, next_move: Color) -> VectorPosition {
        let mut v = vec![];
        for x in cells {
            if let Some(piece) = x {
                let s = if piece.color == Color::Black { -1 } else { 1 };
                let w = if piece.is_king { 3 } else { 1 };
                v.push(s * w);
            } else { v.push(0) }
        }
        v.push(if next_move == Black { -1 } else { 1 });
        VectorPosition(Arc::new(v))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PositionWN {
    pub cells: Vec<Option<Piece>>,
    pub next_move: Color,
    pub W: i64,
    pub N: i64,
    pub NN: Option<i64>,
}

impl PositionWN {
    pub fn fom_node(node: &Node, NN: Option<i64>) -> PositionWN {
        PositionWN {
            W: node.W,
            N: node.N,
            NN,
            cells: node.pos_mov.borrow().pos.cells.clone(),
            next_move: node.pos_mov.borrow().pos.next_move.unwrap(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Node {
    pub W: i64,
    pub N: i64,
    pub average_game_len: f64,
    pub finish: Option<FinishType>,
    pub passed: bool,
    pub(crate) pos_mov: Rc<RefCell<PositionAndMove>>,
    pub childs: HashMap<VectorPosition, Rc<RefCell<Node>>>,
}

impl Node {
    pub fn new(pos_mov: PositionAndMove) -> Node {
        Node {
            W: 0,
            N: 0,
            average_game_len: 0.0,
            finish: None,
            passed: false,
            pos_mov: Rc::new(RefCell::new(pos_mov)),
            childs: HashMap::new(),
        }
    }

    pub fn get_key(&mut self) -> VectorPosition {
        self.pos_mov.borrow_mut().pos.get_key()
    }
    pub fn childs_iter(&self) {}
    pub fn expand(&mut self) {
        let mut base_p = self.pos_mov.borrow().pos.clone();
        let move_list = base_p.get_move_list_cached();
        for mov in &move_list.as_ref().as_ref().unwrap().list {
            let node = Rc::new(
                RefCell::new(Node::new(base_p.make_move_and_get_position(mov))));
            if {
                let child = self.childs.get(&node.borrow_mut().get_key());
                child.is_none() ||
                    child.unwrap().borrow().pos_mov.borrow().mov != node.borrow().pos_mov.borrow().mov
            } {
                self.childs.insert(node.borrow_mut().get_key(), node.clone());
            }
            base_p.unmake_move(mov);
        }
    }
    pub fn get_move(&self) -> Option<MoveItem> {
        self.pos_mov.borrow().mov.clone()
    }
    pub fn get_pos_mov(&self) -> Rc<RefCell<PositionAndMove>> {
        self.pos_mov.clone()
    }
}

#[derive(Serialize, Debug, Clone, Deserialize)]
pub struct OldCacheItem {
    pub node: Arc<Mutex<PositionWN>>,
    pub child: Arc<Mutex<PositionWN>>,
}


#[derive(Serialize)]
pub struct NeuralVecList(Vec<Vec<f32>>);

impl NeuralVecList {
    fn push_many(&mut self, list: NeuralVecList) {
        self.0.extend(list.0);
    }
}

#[derive(Serialize, Debug, Clone, Deserialize)]
pub struct NodeCacheItem {
    key: VectorPosition,
    quality: Quality,
    childs: Vec<(VectorPosition, Quality)>,
}

impl NodeCacheItem {
    pub fn key(&self) -> VectorPosition {
        self.key.clone()
    }

    pub fn from_node(node: &mut Node) -> NodeCacheItem {
        NodeCacheItem {
            key: node.get_key(),
            quality: Quality { N: node.N, W: node.W },
            childs: Vec::from_iter(node.childs.iter().map(|(_, x)| {
                let key = x.borrow_mut().get_key();
                let y = x.borrow();
                (key, Quality { N: y.N, W: y.W })
            })),
        }
    }

    pub fn to_vector_list(&self) -> NeuralVecList {
        let mut v = vec![];
        let mut res = NeuralVecList(vec![]);
        v = self.key.0.iter().map(|x| f32::trunc(*x as f32 / 3.0 * 10.0) / 10.0).collect::<Vec<_>>();
        let next_move = f32::trunc(v.pop().unwrap()*4.0);
        if next_move < 0.0 { v.reverse() }
        for (ve, q) in &self.childs {
            let mut v1 = ve.0.iter().map(|x| f32::trunc(*x as f32 / 3.0 * 10.0) / 10.0).collect::<Vec<_>>();
            let mut v = v.clone();
            if v1.pop().unwrap() < 0.0 { v1.reverse() }
            v.extend(v1);
            v.push(next_move);
            let q_u = 1.4 * f32::sqrt(f32::ln(self.quality.N as f32) / (1.0 + q.N as f32));
            let q_v = (q.W as f32 / (q.N as f32 + 1.0) + 1.0) / 2.0;
            v.push(q_u);
            v.push(q_v);
            res.0.push(v);
        }
        res
    }
}

#[derive(Clone, Deserialize, Serialize, Debug)]
struct Quality {
    pub W: i64,
    pub N: i64,
}

#[derive(Clone, Default)]
pub struct Cache(pub Arc<RwLock<Option<CacheDb<VectorPosition, NodeCacheItem>>>>);

impl Cache {
    pub fn to_file(&self, f_name: String) -> std::io::Result<()> {
        let mut vv = NeuralVecList(vec![]);
        for x in self.0.write().unwrap().as_mut().unwrap().get_map() {
            vv.0.extend(x.get_item().write().unwrap().to_vector_list().0);
        }
        println!("list size {}", vv.0.len());
        let json_data = serde_json::to_string(&vv.0)?;
        let mut file = File::create(f_name)?;
        file.write_all(json_data.as_bytes())?;
        Ok(())
    }
}

// impl Default for Cache {
//     fn default() -> Self {
//         Cache(Arc::new(RwLock::new(
//                 CacheDb::new(CacheItem::key, "checkers".to_string(),
//                              "nodes".to_string(),10_000_000,
//                              10, 1000).await)
//         ))
//     }
// }

pub struct McTree {
    pub root: Rc<RefCell<Node>>,
    history: Rc<RefCell<PositionHistory>>,
    pub cache: Cache,
}


impl McTree {
    pub fn new(pos: Position, history: Rc<RefCell<PositionHistory>>) -> McTree {
        McTree {
            root: Rc::new(RefCell::new(Node {
                W: 0,
                N: 0,
                average_game_len: 0.0,
                finish: None,
                passed: false,
                pos_mov: Rc::new(RefCell::new(PositionAndMove::from_pos(pos))),
                childs: HashMap::new(),
            })),
            history,
            cache:
            Cache(Arc::new(RwLock::new(
                None))),
        }
    }

    pub fn set_cache(&mut self, mut cache: Cache) {
        self.cache = cache;
    }


    pub fn new_from_node(root: Rc<RefCell<Node>>, history: Rc<RefCell<PositionHistory>>, cache: Cache) -> McTree {
        McTree {
            root,
            history,
            cache,
        }
    }

    fn root_search(&self, node: &Rc<RefCell<Node>>, mut max_deps: i16, deps: i16) -> Rc<RefCell<Node>> {
        let color = node.borrow().pos_mov.borrow().pos.next_move.unwrap();
        fn min_max_fn<T, F>(v: &[T], fun: F, color: Color) -> Option<&T>
            where
                F: FnMut(&&T, &&T) -> Ordering
        {
            if color == White { Iterator::max_by(v.iter(), fun) } else { Iterator::max_by(v.iter(), fun) }
        }

        fn vec_pos_move_min_max(l: &[Rc<RefCell<Node>>], color: Color) -> Rc<RefCell<Node>> {
            min_max_fn(l, |x: &&Rc<RefCell<Node>>, y: &&Rc<RefCell<Node>>|
                x.borrow().pos_mov.borrow().pos.state.cmp(
                    &y.borrow().pos_mov.borrow().pos.state), color).unwrap().clone()
        }

        if node.borrow().childs.len() == 0 {
            // if deps==0 {println!("{:?} ", &node.borrow().pos_mov.borrow().mov);}
            return node.clone();
        }
        let l = node.borrow().childs.len();
        let l0 = if l <= 5 { 0 } else { l - 5 };
        if l < 2 { max_deps += 1; }
        let list = &node.borrow().childs.values()
            .map(|x| x.clone()).collect::<Vec<_>>()[l0..l];
        if deps < max_deps {
            let list1 = &list.iter().map(|x|
                self.root_search(x, max_deps, deps + 1)).collect::<Vec<_>>();
            if deps > 0 {
                vec_pos_move_min_max(list1, color)
            } else {
                let node0 = vec_pos_move_min_max(list1, color);
                let mut ind = 0;
                for node in list1 {
                    if node.as_ref().as_ptr() == node0.as_ref().as_ptr() { break; }
                    ind += 1;
                }
                // println!("{:?} ", &list[ind].borrow().pos_mov.borrow().mov);
                list[ind].clone()
            }
        } else {
            vec_pos_move_min_max(list, color)
        }
    }

    pub async fn search(&mut self, max_passes: i32) -> Rc<RefCell<Node>> {
        let mut cached_passes = 0;
        let mut track: Vec<Rc<RefCell<Node>>> = vec![];
        let hist_len = self.history.borrow().len();
        let back_propagation = |mut res: i64, track: &mut Vec<Rc<RefCell<Node>>>,
                                history: &Rc<RefCell<PositionHistory>>,
                                hist_len: usize, cache: &Cache| {
            let mut g_len = 0.0;
            let mut depth = track.len();
            for node in track.iter().rev() {
                depth -= 1;
                let passed = node.borrow().childs.iter().all(|(_, x)| x.borrow().passed);
                node.borrow_mut().passed = passed;
                node.borrow_mut().W += res;
                node.borrow_mut().average_game_len = {
                    let avr = node.borrow().average_game_len;
                    let n = node.borrow().N;
                    (avr * n as f64 + g_len) / (n as f64 + 1.0)
                };

                g_len += 1.0;
                res = -res;
            }
            history.borrow_mut().cut_to(hist_len);
            *track = vec![];
        };
        let mut pass = 0;
        let u = |N: i64, node: &Rc<RefCell<Node>>|
            {
                // let n = node.borrow().childs.iter()
                //     .fold(0, |acc, x| acc + x.borrow().N) as f64;
                2.0 * f64::sqrt(f64::ln((node.borrow().N) as f64) / (N as f64 + 1.0))
                // 2.0 * f64::powi(f64::sqrt(node.borrow().N as f64) / (N as f64 + 1.0), 2)
                // 2.0 * f64::sqrt(
                //     // node.borrow().childs.iter().fold(0, |acc, x|acc+x.borrow().N) as f64
                //     node.borrow().N as f64
                // ) / (N as f64 + 1.0);
                // let u = |N: i64, node: &Rc<RefCell<Node>>|
                //     10.0 * f64::sqrt(node.borrow().N as f64) / (N as f64 + 1.0);
            };
        let u_max = |child: &Node, node: &Rc<RefCell<Node>>| {
            child.W as f64 / (child.N as f64 + 1.0) + u(child.N, node)
        };
        let u_min = |child: &Node, node: &Rc<RefCell<Node>>| {
            // child.N as f64
            child.W as f64 / (child.N as f64 + 1.0) - u(child.N, node)
        };
        let w_n = |a: &Rc<RefCell<Node>>| a.borrow().W as f64 / (1.0 + a.borrow().N as f64);

        let mut update_from_cache = |node: &mut Rc<RefCell<Node>>| {
            // let cache_item = CacheItem::from_node(&mut *node.borrow_mut());
            // let key = cache_item.key();
            let mut ok = false;
            let cache = self.cache.0.read().unwrap();
            let item_val = cache.as_ref().unwrap().get(&node.borrow_mut().get_key());
            if let Some(item) = &item_val {
                cached_passes += 1;
                let it = item.read().unwrap();
                if node.borrow().N < it.quality.N {
                    ok = true;
                    node.borrow_mut().N = it.quality.N;
                    node.borrow_mut().W = it.quality.W;
                    for x in &it.childs {
                        if let Some(child) = node.borrow_mut().childs.get_mut(&x.0) {
                            child.borrow_mut().N = x.1.N;
                            child.borrow_mut().W = x.1.W;
                        }
                    }
                }
                // if node.borrow_mut().N < item.read().unwrap().quality.node.N {
                //     node.borrow_mut().N = item.read().unwrap().quality.node.N;
                //     node.borrow_mut().W = item.read().unwrap().quality.node.W;
                // }
                // if child.borrow_mut().N < item.read().unwrap().quality.child.N {
                //     child.borrow_mut().N = item.read().unwrap().quality.child.N;
                //     child.borrow_mut().W = item.read().unwrap().quality.child.W;
                // }
            }
            ok
        };
        #[derive(Debug)]
        struct Res {
            bw: i32,
            ww: i32,
            d: i32,
        }
        ;
        let mut res: Res = Res { bw: 0, ww: 0, d: 0 };
        while pass < max_passes && self.root.borrow().finish.is_none() {
            let mut node = self.root.clone();
            track.push(node.clone());
            loop {
                pass += 1;
                node.borrow_mut().N += 1;

                update_from_cache(&mut node);

                if node.borrow().N > 100 {
                    let item =
                        self.cache.0.read().unwrap().as_ref().unwrap().get(&node.borrow_mut().get_key());
                    if item.is_none() || node.borrow().N - item.unwrap().read().unwrap().quality.N > 1 {
                        let cache_item = NodeCacheItem::from_node(&mut *node.borrow_mut());
                        self.cache.0.read().unwrap().as_ref().unwrap().insert(cache_item).await;
                    }
                }


                node = {
                    // node.borrow_mut().expand();
                    let move_list = node.borrow_mut().pos_mov.borrow_mut().pos.get_move_list_cached_random_sort();
                    if node.borrow().childs.len() < move_list.as_ref().as_ref().unwrap().list.len() {
                        let i = node.borrow().childs.len();
                        let x = &move_list.as_ref().as_ref().unwrap().list[i];
                        let child = Rc::new(
                            RefCell::new(Node::new(
                                node.borrow().pos_mov.borrow_mut().pos.make_move_and_get_position(x))));
                        node.borrow_mut().childs.insert(child.borrow_mut().get_key(), child.clone());
                        node.borrow().pos_mov.borrow_mut().pos.unmake_move(x);
                        child
                    } else {
                        let b_node = node.borrow();
                        let z_ch: Vec<_> =
                            b_node.childs.values().clone().filter(|x| x.borrow().N < 1).collect();
                        if z_ch.len() > 0 {
                            z_ch[rand::thread_rng().gen_range(0..z_ch.len())].clone()
                        } else {
                            let node_max = b_node.childs.values().max_by(|a, b| {
                                if u_max(&*a.borrow(), &node) < u_max(&*b.borrow(), &node)
                                { Ordering::Less } else { Ordering::Greater }
                            }).unwrap().clone();
                            node_max
                        }
                    }
                };


                let hist_finish = self.history.borrow_mut().push_rc(node.borrow().pos_mov.clone());
                track.push(node.clone());
                // if finish achieved
                if hist_finish.is_none() && node.borrow().finish.is_some() {
                    panic!("finish check error")
                }
                if let Some(finish) = hist_finish
                // {
                // if hist_finish.is_some() { hist_finish.clone() } else {
                //     if node.borrow().finish.is_some() { node.borrow().finish.clone() } else {
                //         None
                //     }
                // }
                {
                    node.borrow_mut().finish = Some(finish.clone());
                    node.borrow_mut().passed = true;
                    back_propagation({
                                         let fr = match finish {
                                             FinishType::WhiteWin => {
                                                 res.ww += 1;
                                                 1
                                             }
                                             FinishType::BlackWin => {
                                                 res.bw += 1;
                                                 -1
                                             }
                                             _ => {
                                                 res.d += 1;
                                                 0
                                             }
                                         };
                                         let first =
                                             if node.borrow_mut().pos_mov.borrow().pos.next_move
                                                 == Some(White) { -1 } else { 1 };
                                         // let par =
                                         //     if track.len() % 2 == 0 { 1 } else { -1 };
                                         fr * first
                                     }, &mut track, &self.history, hist_len, &self.cache);
                    break;
                }
            }
        }
        // println!("cached: {}", cached_passes);
        if self.root.borrow().finish.is_some() {
            panic!("finish achieved")
        }
        if self.root.borrow().childs.len() > 0 {
            println!("_______");


            let best = self.root.borrow().childs.values().max_by(|a, b|
                // a.borrow().N.cmp(&b.borrow().N)
                // if u(a.borrow().N, a.borrow().NN, &node) < u(b.borrow().N, b.borrow().NN, &node) {
                //     Ordering::Less
                // } else {
                //     Ordering::Greater
                // }

                if u_min(&a.borrow(), &self.root) <
                    u_min(&b.borrow(), &self.root) { Ordering::Less } else { Ordering::Greater }
            ).unwrap().clone();
            println!("{:?} {:?} {} {} {:?}", thread::current().id(),
                     if self.history.borrow().list.len() % 2 == 0 { Color::White } else { Black },
                     best.borrow().W as f64 / (best.borrow().N as f64 + 1.0), u(best.borrow().N, &self.root), res);
            best
        } else {
            panic!("no childs")
        }
    }

    // pub fn get_cache_json (&self) -> String {
    //     // self.cache.serialize().unwrap()
    // }

    pub fn root_map(&self) -> Vec<i64> {
        self.root.borrow().childs.iter().map(|(_, x)| x.borrow().N).collect::<Vec<_>>()
    }

    pub fn tree_childs(&self) -> Vec<Rc<RefCell<Node>>> {
        self.root.borrow().childs.values().map(|x| x.clone()).collect::<Vec<_>>()
    }
}
use std::cell::{Ref, RefCell};
use std::cmp::Ordering;
use std::rc::Rc;
use js_sys::Math::sign;
use crate::position::{Position, PositionKey};
use crate::PositionHistory::{FinishType, PositionAndMove, PositionHistory};
use rand::{Rng};
use rayon::prelude::IntoParallelRefIterator;
use serde::Serialize;
use crate::{color, log};
use crate::cache_map::{CacheMap, Wrapper};
use crate::color::Color;
use crate::color::Color::{Black, White};
use crate::moves_list::MoveItem;
use crate::piece::Piece;
use serde::Deserialize;
use rayon::prelude::*;
use std::iter::Iterator;
use std::sync::{Arc, Mutex};


#[derive(Debug, Serialize, Deserialize)]
pub struct PositionWN {
    pub cells: Vec<Option<Piece>>,
    pub next_move: Color,
    pub W: i64,
    pub N: i64,
}

impl PositionWN {
    pub fn fom_node(node: &Node) -> PositionWN {
        PositionWN {
            W: node.W,
            N: node.N,
            cells: node.pos_mov.borrow().pos.cells.clone(),
            next_move: node.pos_mov.borrow().pos.next_move.unwrap(),
        }
    }

    pub fn map_key(&self) -> (Vec<Option<Piece>>, Option<Color>) {
        (self.cells.clone(), Some(self.next_move))
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
    pub childs: Vec<Rc<RefCell<Node>>>,
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
            childs: vec![],
        }
    }
    pub fn expand(&mut self) {
        if self.childs.len() > 0 { return; }
        let mut base_p = self.pos_mov.borrow().pos.clone();
        let move_list = base_p.get_move_list_cached();
        // let childs = Arc::new(Mutex::new(vec![]));
        // move_list.as_ref().as_ref().unwrap().list.par_iter().for_each(|x| {
        //     childs.lock().unwrap().push(base_p.clone().make_move_and_get_position(x))
        // });
        // let childs = childs.lock().unwrap().clone();
        // for x in childs {
        //     self.childs.push(Rc::new(RefCell::new(Node::new(x))))
        // }
        for mov in &move_list.as_ref().as_ref().unwrap().list {
            self.childs.push(Rc::new(
                RefCell::new(Node::new(base_p.make_move_and_get_position(mov)))));
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


pub type Cache = Arc<Mutex<CacheMap<PositionKey, Arc<Mutex<PositionWN>>>>>;

#[derive(Debug)]
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
                childs: vec![],
            })),
            history,
            cache:
            Arc::new(Mutex::new(
                CacheMap::new(|pos_wn: &Arc<Mutex<PositionWN>>| pos_wn.lock().unwrap().map_key(), 1_000_000))),
        }
    }

    pub fn set_cache(&mut self, cache: Cache) {
        self.cache = cache;
    }


    pub fn new_from_node(root: Rc<RefCell<Node>>, history: Rc<RefCell<PositionHistory>>, cache:
    Arc<Mutex<CacheMap<PositionKey, Arc<Mutex<PositionWN>>>>>) -> McTree {
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
        let list = &node.borrow().childs[l0..l];
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

    pub fn search(&mut self, max_passes: i32) -> Rc<RefCell<Node>> {
        let mut track: Vec<Rc<RefCell<Node>>> = vec![];
        let hist_len = self.history.borrow().len();
        let back_propagation = |mut res: i64, track: &mut Vec<Rc<RefCell<Node>>>,
                                history: &Rc<RefCell<PositionHistory>>,
                                hist_len: usize, cache: Arc<Mutex<CacheMap<PositionKey, Arc<Mutex<PositionWN>>>>>| {
            let mut g_len = 0.0;
            let mut depth = track.len();
            for node in track.iter().rev() {
                depth -= 1;
                let passed = node.borrow().childs.iter().all(|x| x.borrow().passed);
                node.borrow_mut().passed = passed;
                node.borrow_mut().W += res;
                node.borrow_mut().average_game_len = {
                    let avr = node.borrow().average_game_len;
                    let n = node.borrow().N;
                    (avr * n as f64 + g_len) / (n as f64 + 1.0)
                };
                if depth < 3 {
                    let key = node.borrow().pos_mov.borrow().pos.map_key();
                    if node.borrow().N > (max_passes / (i32::pow(5, depth as u32))) as i64 / 2 {
                        let ch_node = cache.lock().unwrap().get(&key);
                        if ch_node.is_none() || (ch_node.is_some() &&
                            ch_node.unwrap().lock().unwrap().item.lock().unwrap().N < node.borrow().N) {
                            cache.lock().unwrap()
                                .insert(Arc::new(Mutex::new(PositionWN::fom_node(&node.borrow()))));
                        }
                    }
                }
                g_len += 1.0;
                res = -res;
            }
            history.borrow_mut().cut_to(hist_len);
            *track = vec![];
        };
        let mut pass = 0;
        let u = |N: i64, node: &Rc<RefCell<Node>>|
            {
                let n = node.borrow().childs.iter()
                    .fold(0, |acc, x| acc + x.borrow().N) as f64;
                1.4 * f64::sqrt(f64::ln(n) / (N as f64 + 1.0))

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
        while pass < max_passes && self.root.borrow().finish.is_none() {
            let mut node = self.root.clone();
            loop {
                node.borrow_mut().N += 1;
                node.borrow_mut().expand();
                let childs = node.borrow().childs.clone();
                // childs = childs.iter().map(|chi| {
                //     let key = chi.borrow().pos_mov.borrow().pos.map_key();
                //     let pos_wn = self.cache.borrow_mut().get(&key);
                //     if let Some(pos_wn) = pos_wn {
                //         let dw = pos_wn.borrow().item.borrow().W - chi.borrow_mut().W;
                //         let dn = pos_wn.borrow().item.borrow().N - chi.borrow_mut().N;
                //         if dn > 0 {
                //             chi.borrow_mut().W += dw;
                //             chi.borrow_mut().N += dn;
                //             print!(" {} ", dn);
                //             let mut sig = -1;
                //             track.iter().for_each(|x| {
                //                 x.borrow_mut().N += dn - 1;
                //                 x.borrow_mut().W += (sig * dw);
                //                 sig = -sig;
                //             })
                //         }
                //     }
                //     chi.clone()
                // }).collect::<Vec<_>>();
                pass += 1;
                node = {
                    childs.iter().for_each(|x| {
                        if x.borrow().N == 0 {
                            let key = x.borrow().pos_mov.borrow().pos.map_key();
                            let pos_wn = self.cache.as_ref().lock().unwrap().get(&key);
                            if let Some(pos_wn) = &pos_wn {
                                x.borrow_mut().N = pos_wn.lock().unwrap().item.lock().unwrap().N;
                                x.borrow_mut().W = pos_wn.lock().unwrap().item.lock().unwrap().W;
                            }
                        }
                    });
                    let z_ch: Vec<_> = childs.iter().filter(|x| x.borrow().N == 0).collect();
                    if z_ch.len() > 0 {
                        z_ch[rand::thread_rng().gen_range(0..z_ch.len())].clone()
                    } else {
                        let node_max = childs.iter().max_by(|a, b| {
                            if u_max(&*a.borrow(), &node) < u_max(&*b.borrow(), &node)
                            { Ordering::Less } else { Ordering::Greater }
                        }).unwrap().clone();
                        node_max

                        // let umax = u_max(&*node_max.borrow(), &node);
                        // let eq_nodes = childs.iter().filter(|x| {
                        //     let x_umax = u_max(&*x.borrow(), &node);
                        //     // ~5% -> equal
                        //     f64::abs((umax - x_umax) / (umax + x_umax)) < 0.0025
                        // });
                        // if eq_nodes.clone().collect::<Vec<_>>().len() > 1 {
                        //     print!("{:?} {:?}\n", eq_nodes.clone().collect::<Vec<_>>().len(), node.borrow().average_game_len);
                        // }
                        // let eq_list = eq_nodes.collect::<Vec<_>>();
                        // eq_list[rand::thread_rng().gen_range(0..eq_list.len())].clone()
                        // if node_max.borrow().W > 0 {
                        //     eq_nodes.min_by(|x,y|
                        //         (x.borrow().average_game_len).total_cmp(&y.borrow().average_game_len)).unwrap().clone()
                        // } else {
                        //     eq_nodes.max_by(|x,y|
                        //         (x.borrow().average_game_len).total_cmp(&y.borrow().average_game_len)).unwrap().clone()
                        // }


                        //
                        // node.borrow_mut().childs = childs;
                        // node.borrow().childs.last().unwrap().clone()
                    }
                };

                let hist_finish = self.history.borrow_mut().push_rc(node.borrow().pos_mov.clone());
                track.push(node.clone());
                // if finish achieved
                if let Some(finish) = {
                    if hist_finish.is_some() { hist_finish.clone() } else {
                        if node.borrow().finish.is_some() { node.borrow().finish.clone() } else {
                            None
                        }
                    }
                } {
                    node.borrow_mut().finish = Some(finish.clone());
                    node.borrow_mut().passed = true;
                    back_propagation({
                                         let fr = if finish == FinishType::WhiteWin { 1 } else if
                                         finish == FinishType::BlackWin { -1 } else { 0 };
                                         let sing =
                                             if node.borrow().pos_mov.borrow().pos.next_move.unwrap() == Color::White { -1 } else { 1 };
                                         fr * sing
                                     }, &mut track, &self.history, hist_len, self.cache.clone());
                    break;
                }
            }
        }
        if self.root.borrow().finish.is_some() {
            panic!("finish achieved")
        }
        if self.root.borrow().childs.len() > 0 {
            let node = self.root.clone();
            self.root.borrow().childs.iter().max_by(|a, b|
                // if u_max(&*a.borrow(), &node) < u_max(&*b.borrow(), &node)
                // { Ordering::Less } else { Ordering::Greater }).unwrap().clone()
                if u_min(&a.borrow(), &self.root) <
                    u_min(&b.borrow(), &self.root) { Ordering::Less } else { Ordering::Greater }).unwrap().clone()
        } else {
            panic!("no childs")
        }
    }

    // pub fn get_cache_json (&self) -> String {
    //     // self.cache.serialize().unwrap()
    // }

    pub fn root_map(&self) -> Vec<i64> {
        self.root.borrow().childs.iter().map(|x| x.borrow().N).collect::<Vec<_>>()
    }

    pub fn tree_childs(&self) -> Vec<Rc<RefCell<Node>>> {
        self.root.borrow().childs.clone()
    }
}
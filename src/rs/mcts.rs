use std::cell::RefCell;
use std::cmp::Ordering;
use std::rc::Rc;
use crate::position::{Position};
use crate::PositionHistory::{FinishType, PositionAndMove, PositionHistory};
use rand::{Rng};
use schemars::_private::NoSerialize;
use crate::color;
use crate::color::Color;
use crate::color::Color::{Black, White};
use crate::moves_list::MoveItem;

#[derive(Debug)]
pub struct Node {
    pub W: i64,
    pub N: i64,
    passed_completely: bool,
    pos_mov: Rc<RefCell<PositionAndMove>>,
    pub childs: Vec<Rc<RefCell<Node>>>,
}

impl Node {
    pub fn new(pos_mov: PositionAndMove) -> Node {
        Node {
            W: 0,
            N: 0,
            passed_completely: false,
            pos_mov: Rc::new(RefCell::new(pos_mov)),
            childs: vec![],
        }
    }
    pub fn expand(&mut self) {
        if self.childs.len() > 0 { return; }
        let mut base_p = self.pos_mov.borrow().pos.clone();
        let move_list = base_p.get_move_list_cached();
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

#[derive(Debug)]
pub struct McTree {
    root: Rc<RefCell<Node>>,
    history: Rc<RefCell<PositionHistory>>,
}

impl McTree {
    pub fn new(pos: Position, history: Rc<RefCell<PositionHistory>>) -> McTree {
        McTree {
            root: Rc::new(RefCell::new(Node {
                W: 0,
                N: 0,
                passed_completely: false,
                pos_mov: Rc::new(RefCell::new(PositionAndMove::from_pos(pos))),
                childs: vec![],
            })),
            history,
        }
    }

    pub fn new_from_node(root: Rc<RefCell<Node>>, history: Rc<RefCell<PositionHistory>>) -> McTree {
        McTree {
            root,
            history,
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
    pub fn search(&mut self, max_passes: i32) -> Option<Rc<RefCell<Node>>> {
        let mut track: Vec<Rc<RefCell<Node>>> = vec![];
        let hist_len = self.history.borrow().len();
        fn back_propagation(mut res: i64, track: &mut Vec<Rc<RefCell<Node>>>,
                            history: &Rc<RefCell<PositionHistory>>, hist_len: usize) {
            if res != 0 {
                for node in track.iter().rev() {
                    node.borrow_mut().W += res;
                    res = -res;
                }
            }
            history.borrow_mut().cut_to(hist_len);
            *track = vec![];
        }
        let mut pass = 0;
        while pass < max_passes && !self.root.borrow().passed_completely {
            let mut node = self.root.clone();
            loop {
                node.borrow_mut().N += 1;
                node.borrow_mut().expand();
                let u = |child: &Node|
                    1.4 * f64::sqrt(f64::ln(node.borrow().N as f64) / (child.N as f64 + 1.0));
                let u_max = |child: &Node| {
                    child.W as f64 / (child.N as f64 + 1.0) + u(child) + {
                        0.0
                        // if child.pos_mov.as_ref().borrow().mov.as_ref().unwrap().strike.is_some() { 1000.0 } else { 0.0 }
                    }
                };
                let mut childs = node.borrow().childs.clone();
                // for child in &node.borrow().childs {
                //     if !child.borrow().passed_completely { childs.push(child.clone()); }
                // }
                if childs.len() > 0 {
                    pass += 1;
                    node = {
                        let z_ch: Vec<_> = childs.iter().filter(|x| x.borrow().N == 0).collect();
                        if z_ch.len() > 0 {
                            z_ch[rand::thread_rng().gen_range(0..z_ch.len())].clone()
                        } else {
                            childs.sort_by(|a, b|
                                if u_max(&*a.borrow()) < u_max(&*b.borrow())
                                { Ordering::Less } else { Ordering::Greater });
                            let n_max = node.borrow().N;
                            if n_max as f64 > 500000.0 / f64::ln(track.len() as f64) * (childs.len() as f64) {
                                let c_max = node.borrow().childs.iter()
                                    .max_by(|x, y|
                                        u_max(&x.borrow()).total_cmp(&u_max(&y.borrow())))
                                    .unwrap().borrow().N;
                                let mut new_ch = vec![];
                                for child in &childs {
                                    if (c_max as f64 / child.borrow().N as f64) < 4.5 {
                                        new_ch.push(child.clone());
                                    }
                                }
                                if new_ch.len() > 0 {
                                    childs = new_ch;
                                }
                            }
                            node.borrow_mut().childs = childs;
                            node.borrow().childs.last().unwrap().clone()
                        }
                    };
                } else {
                    node.borrow_mut().passed_completely = true;
                    track.push(node);
                    back_propagation(track.pop().unwrap().borrow().W, &mut track, &self.history, hist_len);
                    break;
                }

                track.push(node.clone());
                let finish = self.history.borrow_mut().push_rc(node.borrow().pos_mov.clone());
                // if finish achieved
                if let Some(finish) = finish {
                    node.borrow_mut().passed_completely = true;
                    back_propagation({
                                         let fr = if finish == FinishType::WhiteWin { 1 } else if
                                         finish == FinishType::BlackWin { -1 } else { 0 };
                                         let sing =
                                             if node.borrow().pos_mov.borrow().pos.next_move.unwrap() == Color::White { -1 } else { 1 };
                                         fr * sing
                                     }, &mut track, &self.history, hist_len);
                    break;
                }
            }
        }
        if self.root.borrow().childs.len() > 0 {
            // let chs: Vec<_> =
            //     self.root.borrow().childs.iter().map(|x| (x.borrow().W, x.borrow().N)).collect();
            // print!(" ch: {:?} \n", chs);
            Some(self.root.borrow().childs.iter().max_by(|x, y| x.borrow().N.cmp(&y.borrow().N)).unwrap().clone())
            // Some(self.root.borrow().childs.last().unwrap().clone())
            // Some(self.root_search(&self.root, 10, 0))
        } else {
            None
        }
    }

    pub fn root_map(&self) -> Vec<i64> {
        self.root.borrow().childs.iter().map(|x|x.borrow().N).collect::<Vec<_>>()
    }

    pub fn tree_childs(&self) -> Vec<Rc<RefCell<Node>>> {
        self.root.borrow().childs.clone()
    }
}
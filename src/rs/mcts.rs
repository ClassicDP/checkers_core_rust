use std::cell::RefCell;
use std::cmp::Ordering;
use std::io;
use std::io::Write;
use std::ops::Deref;
use std::rc::Rc;
use crate::position::Position;
use crate::PositionHistory::{FinishType, PositionAndMove, PositionHistory};
use rand::{Rng};
use schemars::_private::NoSerialize;
use crate::color::Color;
use crate::moves_list::MoveItem;

#[derive(Debug)]
pub struct Node {
    W: i64,
    N: i64,
    passed_completely: bool,
    pos_mov: Rc<RefCell<PositionAndMove>>,
    childs: Vec<Rc<RefCell<Node>>>,
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
                    1.4*f64::sqrt(f64::ln(node.borrow().N as f64) / (child.N as f64 + 1.0));
                let u_max = |child: &Node| child.W as f64 / (child.N as f64 + 1.0) + u(child);
                let mut childs = node.borrow().childs.clone();
                // for child in &node.borrow().childs {
                //     if !child.borrow().passed_completely { childs.push(child.clone()); }
                // }
                if childs.len() > 0 {
                    node = {
                        let z_ch: Vec<_> = childs.iter().filter(|x| x.borrow().N == 0).collect();
                        if z_ch.len() > 0 {
                            z_ch[rand::thread_rng().gen_range(0..z_ch.len())].clone()
                        } else {
                            childs.sort_by(|a, b|
                                if u_max(&*a.borrow()) < u_max(&*b.borrow())
                                { Ordering::Less } else { Ordering::Greater });
                            let n_max = node.borrow().N;
                            // if n_max > (5 * childs.len()) as i64 {
                            //     let n_per_mov = n_max as i64 / childs.len() as i64;
                            //     let mut new_ch = vec![];
                            //     for child in &childs {
                            //         if (n_per_mov as f64/ child.borrow().N as f64) < 1.3 {
                            //             new_ch.push(child.clone());
                            //         }
                            //     }
                            //     if new_ch.len() > 0 {
                            //         childs = new_ch;
                            //     }
                            // }
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
            pass += 1;
        }
        if self.root.borrow().childs.len() > 0 {
           Some(self.root.borrow().childs.last().unwrap().clone())
        } else {
            None
        }
    }
}
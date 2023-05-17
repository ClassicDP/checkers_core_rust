use std::cell::{RefCell};
use std::rc::Rc;
use std::sync::Arc;
use wasm_bindgen::prelude::*;
use crate::color::Color;
use crate::moves::BoardPos;
use crate::moves_list::{MoveItem, MoveList};
use crate::piece::Piece;
use crate::position::{Position};
use crate::position_environment::PositionEnvironment;
use ts_rs::*;
use serde::Serialize;
use crate::color::Color::{Black, White};
use crate::game::Method::{Deep, MCTS};
use crate::PositionHistory::FinishType::{BlackWin, Draw1, Draw2, Draw3, Draw4, Draw5, WhiteWin};
use crate::mcts::{McTree, Node};
use crate::PositionHistory::{FinishType, PositionAndMove, PositionHistory};

#[wasm_bindgen]
pub struct MCTSRes {
    #[wasm_bindgen(skip)]
    pub board_list: Option<Vec<Vec<i32>>>,
    #[wasm_bindgen(skip)]
    pub finish: Option<FinishType>,
    #[wasm_bindgen(skip)]
    pub pos_move: Option<Rc<RefCell<PositionAndMove>>>,
}

#[wasm_bindgen]
#[derive(Serialize, Debug)]
#[derive(TS)]
#[ts(export)]
pub struct BestPos {
    pos: Option<Rc<RefCell<PositionAndMove>>>,
    pos_list: Vec<Rc<RefCell<PositionAndMove>>>,
    deep_eval: i32,
}

impl BestPos {
    pub fn get_move_item(&self) -> MoveItem {
        self.pos.as_ref().unwrap().borrow().mov.as_ref().unwrap().clone()
    }
}

#[wasm_bindgen]
pub enum Method {
    Deep,
    MCTS,
    Mix,
}

#[wasm_bindgen]
pub struct Game {
    #[wasm_bindgen(skip)]
    pub position_history: Rc<RefCell<PositionHistory>>,
    position_environment: Arc<PositionEnvironment>,
    #[wasm_bindgen(skip)]
    pub current_position: Position,
    max_depth: i16,
    mcts_lim: i32,
    method: Method,
    #[wasm_bindgen(skip)]
    pub tree: Option<McTree>,
}

#[wasm_bindgen]
impl Game {
    #[wasm_bindgen(constructor)]
    pub fn new(size: i8) -> Self {
        let environment = Arc::new(PositionEnvironment::new(size));
        let position = Position::new(environment.clone());
        let position_history = Rc::new(RefCell::new(PositionHistory::new()));
        Game {
            position_environment: environment.clone(),
            current_position: position.clone(),
            position_history,
            max_depth: 3,
            method: Deep,
            mcts_lim: 10000,
            tree: None
        }
    }

    #[wasm_bindgen]
    pub fn set_depth(&mut self, depth: i16) {
        self.max_depth = depth;
    }

    #[wasm_bindgen]
    pub fn set_mcts_lim(&mut self, mcts_lim: i32) {
        self.mcts_lim = mcts_lim;
    }

    #[wasm_bindgen]
    pub fn set_method(&mut self, method: Method) {
        self.method = method;
    }

    #[wasm_bindgen]
    pub fn insert_piece(&mut self, piece: Piece) {
        self.current_position.insert_piece(piece);
    }

    #[wasm_bindgen]
    pub fn remove_piece(&mut self, pos: BoardPos) -> bool {
        self.current_position.remove_piece(pos)
    }


    #[wasm_bindgen(getter)]
    pub fn position(&self) -> JsValue {
        match serde_wasm_bindgen::to_value(&self.current_position) {
            Ok(js) => js,
            Err(_err) => JsValue::UNDEFINED,
        }
    }

    pub fn make_move_by_pos_item(&mut self, pos: &BestPos) {
        self.current_position.make_move(&mut pos.get_move_item());
        self.position_history.borrow_mut().push(PositionAndMove::from(self.current_position.clone(), pos.get_move_item()));
    }

    pub fn make_move_by_move_item(&mut self, move_item: &MoveItem) -> Option<FinishType> {
        self.current_position.make_move(move_item);
        self.position_history.borrow_mut()
            .push(PositionAndMove::from(self.current_position.clone(), move_item.clone()))
    }

    #[wasm_bindgen]
    pub fn best_move(&mut self, mut max_depth: i16, mut best_white: i32,
                     mut best_black: i32, depth: i16, state_only: bool) -> BestPos {
        // log(&format!("{:?}", self.current_position));
        let finish = self.position_history.borrow_mut().finish_check();
        if finish.is_some() {
            // print!("{:?} {}\n", finish, depth);
            // pos_it.position.print_pos();
            let pos_it = self.position_history.borrow_mut().last();
            let eval = pos_it.borrow_mut().pos.evaluate(state_only);
            pos_it.borrow_mut().deep_eval = Option::from(eval);
            return BestPos { deep_eval: eval, pos_list: vec![pos_it.clone()], pos: Option::from(pos_it) };
        }
        let ref move_list = self.current_position.get_move_list_cached();
        let mut pos_list: Vec<_> = {
            move_list.as_ref().as_ref().unwrap().list.iter().map(|x| {
                let mut pos = self.current_position.make_move_and_get_position(x);
                pos.pos.evaluate(state_only);
                self.current_position.unmake_move(x);
                Rc::new(RefCell::new(pos))
            }).collect()
        };
        if pos_list.len() == 0 { panic!("Best move: it`s standoff position") }
        let move_color = self.current_position.next_move.unwrap();
        if pos_list.len() < 3 { max_depth += 1; }
        pos_list.sort_by_key(|x|
            x.borrow().pos.eval.unwrap() * if move_color == White { -1 } else { 1 });
        let res_pos_list = pos_list.clone();
        let mut best_pos = BestPos {
            pos: None,
            pos_list: res_pos_list.clone(),
            deep_eval: if move_color == White { i32::MIN / 2 } else { i32::MAX / 2 },
        };
        if depth < max_depth {
            for pos_it in &pos_list {
                self.current_position.make_move(pos_it.borrow().mov.as_ref().unwrap());
                self.position_history.borrow_mut().push_rc(pos_it.clone());
                let deep_eval =
                    self.best_move(max_depth, best_white, best_black, depth + 1, state_only).deep_eval;
                let pos_it = self.position_history.borrow_mut().pop().unwrap();
                pos_it.borrow_mut().deep_eval = Option::from(deep_eval);
                self.current_position.took_pieces = pos_it.borrow().pos.took_pieces.clone();
                self.current_position.unmake_move(&pos_it.borrow().mov.as_ref().unwrap());
                let white = self.current_position.state.white.clone();
                let black = self.current_position.state.black.clone();
                self.current_position.state = pos_it.borrow().pos.state.clone();
                self.current_position.state.white = white;
                self.current_position.state.black = black;
                if move_color == White {
                    if best_black < deep_eval {
                        // print!("cut at white move depth: {} {} {} {}\n", depth, best_black, best_white, deep_eval);
                        return BestPos { pos: Some(pos_it), pos_list: res_pos_list.clone(), deep_eval };
                    }
                    if best_white < deep_eval { best_white = deep_eval }
                    if best_pos.deep_eval < deep_eval {
                        best_pos = BestPos { pos: Option::from(pos_it), pos_list: res_pos_list.clone(), deep_eval };
                    }
                } else {
                    if best_white > deep_eval {
                        // print!("cut at black move depth: {} {} {} {}\n", depth, best_black, best_white, deep_eval);
                        return BestPos { pos: Option::from(pos_it), pos_list: res_pos_list.clone(), deep_eval };
                    }
                    if best_black > deep_eval { best_black = deep_eval }
                    if best_pos.deep_eval > deep_eval {
                        best_pos = BestPos { pos: Option::from(pos_it), pos_list: res_pos_list.clone(), deep_eval };
                    }
                }
            }
        } else {
            let pos = pos_list.pop().unwrap();
            let eval = pos.borrow_mut().pos.evaluate(state_only);
            best_pos = BestPos {
                deep_eval: eval,
                pos_list: res_pos_list.clone(),
                pos: Some(pos),
            }
        }
        best_pos
    }

    pub fn mix_method(&mut self, apply: bool) -> BestPos {
        let mut best_move =
            self.best_move(self.max_depth, i32::MIN / 2, i32::MAX / 2, 0, true);
        let move_color = self.current_position.next_move.unwrap();
        if best_move.pos_list.iter().any(|x| x.borrow().deep_eval.is_none()) {
            print!("strange list: {:?}\n",
                   best_move.pos_list.iter().map(|x| x.borrow().deep_eval).collect::<Vec<_>>());
        }
        best_move.pos_list.sort_by_key(|x|
            x.borrow().deep_eval.unwrap());


        if move_color == White {
            best_move.pos_list.reverse();
        }
        let eval_max_min = best_move.pos_list.first().unwrap().borrow().deep_eval.unwrap();
        print!("eval: {}\n", eval_max_min);
        let mut pos_list = vec![];
        for pos in best_move.pos_list.iter() {
            let condition = i32::abs(eval_max_min - pos.borrow().deep_eval.unwrap()) > 5000;
            if condition {
                print!("break cond: {} {} {:?}\n", eval_max_min, pos.borrow().deep_eval.unwrap(), pos_list.len());
                break;
            }
            pos_list.push(pos.clone());
        }

        self.preparing_tree();

        self.tree.as_mut().unwrap().root.borrow_mut().childs = {
            let mut ch_list = vec![];
            for child in self.tree.as_ref().unwrap().root.borrow().childs.clone() {
                if pos_list.iter().any(|x| x.borrow_mut().pos == child.borrow().pos_mov.borrow().pos)
                { ch_list.push(child.clone()) }
            }
            ch_list
        };
        print!("ch: {}\n", self.tree.as_mut().unwrap().root.borrow_mut().childs.len());
        self.find_mcts_and_make_best_move(false);

        let node = self.tree.as_ref().unwrap().tree_childs().iter().max_by(|x, y|
            x.borrow().W.cmp(&y.borrow().W)).unwrap().clone();


        let best_move = BestPos {
            pos: Option::from(node.borrow().pos_mov.clone()),
            pos_list: vec![],
            deep_eval: 0,
        };
        if apply {
            self.apply_node_move(node);
        }
        best_move
    }

    #[wasm_bindgen]
    pub fn get_or_apply_best_move(&mut self, apply: bool) -> JsValue {
        let finish = self.position_history.borrow_mut().finish_check();
        if finish.is_some() {
            return match serde_wasm_bindgen::to_value(&finish.unwrap()) {
                Ok(js) => js,
                Err(_err) => JsValue::UNDEFINED
            };
        }
        let best_move = match self.method {
            Deep => {
                let best_move = self.best_move(self.max_depth, i32::MIN / 2, i32::MAX / 2, 0, false);
                if apply {
                    self.make_best_move(&best_move);
                }
                best_move
            }
            MCTS => {
                let best = self.find_mcts_and_make_best_move(apply);
                BestPos { pos: best.pos_move, pos_list: vec![], deep_eval: 0 }
            }
            Method::Mix => {
                self.mix_method(apply)
            }
        };
        match serde_wasm_bindgen::to_value(
            &best_move) {
            Ok(js) => js,
            Err(_err) => JsValue::UNDEFINED
        }
    }

    #[wasm_bindgen]
    pub fn make_best_move(&mut self, pos: &BestPos) {
        // log(&format!("{:?}", pos));
        self.make_move_by_pos_item(pos);
    }

    // for neural
    fn get_board_list(&mut self) -> Vec<Vec<i32>> {
        let move_list = self.current_position.get_move_list_cached();
        let mut pos_list: Vec<PositionAndMove> = vec![];
        for ref mut mov in move_list.as_ref().as_ref().unwrap().list.clone() {
            pos_list.push(self.current_position.make_move_and_get_position(mov));
            self.current_position.unmake_move(mov);
        }
        let mut board_list: Vec<Vec<i32>> = vec![];
        for pos in pos_list {
            let mut board = vec![0; (self.position_environment.size * self.position_environment.size / 2) as usize];
            for cell in &pos.pos.cells {
                if let Some(piece) = cell {
                    board[piece.pos] =
                        (if piece.is_king { 3 } else { 1 }) * if piece.color == Color::White { 1 } else { -1 }
                }
            }
            // board.push(pos.pos.evaluate());
            board_list.push(board);
        }
        board_list
    }
    #[wasm_bindgen]
    pub fn find_and_make_best_move_ts_n(&mut self) -> JsValue {
        let finish = self.position_history.borrow_mut().finish_check();
        if finish.is_some() {
            return match serde_wasm_bindgen::to_value(&finish.unwrap()) {
                Ok(js) => js,
                Err(_err) => JsValue::UNDEFINED
            };
        }
        let best_pos = self.best_move(self.max_depth, i32::MIN / 2, i32::MAX / 2, 0, false);
        self.make_move_by_pos_item(&best_pos);
        let finish = self.position_history.borrow_mut().finish_check();
        if finish.is_some() {
            return match serde_wasm_bindgen::to_value(&finish.unwrap()) {
                Ok(js) => js,
                Err(_err) => JsValue::UNDEFINED
            };
        }

        return match serde_wasm_bindgen::to_value(&self.get_board_list()) {
            Ok(js) => js,
            Err(_err) => JsValue::UNDEFINED
        };
    }

    #[wasm_bindgen]
    pub fn find_mcts_and_make_best_move_ts_n(&mut self, apply: bool) -> JsValue {
        let res: MCTSRes = self.find_mcts_and_make_best_move(apply);
        return if res.board_list.is_some() {
            match serde_wasm_bindgen::to_value(&res.board_list.unwrap()) {
                Ok(js) => js,
                Err(_err) => JsValue::UNDEFINED
            }
        } else {
            match serde_wasm_bindgen::to_value(&res.finish.unwrap()) {
                Ok(js) => js,
                Err(_err) => JsValue::UNDEFINED
            }
        };
    }

    fn apply_node_move(&mut self, node: Rc<RefCell<Node>>) {
        self.current_position = node.clone().borrow().pos_mov.borrow().pos.clone();
        self.position_history.borrow_mut().push_rc(
            node.clone().borrow().pos_mov.clone());
        self.tree = Option::from(McTree::new_from_node(node.clone(), self.position_history.clone(),
                                                       self.tree.as_mut().unwrap().cache.clone()));
    }


    pub fn init_tree(&mut self) {
        if self.tree.is_none() {
            self.tree = Some(McTree::new(self.current_position.clone(), self.position_history.clone()));
        }
    }


    pub fn check_tree_for_finish(&mut self) -> Option<MCTSRes> {
        self.init_tree();
        if self.tree.as_ref().unwrap().root.borrow().finish.is_some() {
            return Some(MCTSRes { finish: self.tree.as_ref().unwrap().root.borrow().finish.clone(), board_list: None, pos_move: None });
        }
        None
    }


    pub fn preparing_tree(&mut self) {
        self.init_tree();
        if self.tree.as_ref().unwrap().root.borrow().pos_mov.borrow().pos != self.current_position {
            if let Some(tree) = &self.tree {
                tree.root.borrow_mut().expand();
                let chs = tree.tree_childs();
                let node =
                    chs.iter().find(|x| x.borrow().pos_mov.borrow().pos == self.current_position);
                if node.is_some() {
                    self.tree.as_mut().unwrap().root = node.unwrap().clone();
                } else {
                    // print!("{:?}\n", tree.tree_childs().iter_mut().map(|x|x.borrow().pos_mov.clone()));
                    let chs = tree.tree_childs();
                    for ch in chs {
                        print!("-------\n");
                        for x in ch.borrow().pos_mov.borrow().pos.cells.iter().enumerate() {
                            if self.current_position.cells[x.0] != *x.1 {
                                print!("{:?}  ", x.1);
                            }
                        }
                        print!("\n");
                    }
                    print!("{:?}\n", self.current_position);
                    panic!("node error")
                }
            }
        }
        self.tree.as_mut().unwrap().root.borrow_mut().expand();
    }

    #[wasm_bindgen]
    pub fn find_mcts_and_make_best_move(&mut self, apply: bool) -> MCTSRes {

        self.preparing_tree();
        // let finish = self.check_tree_for_finish();
        // if finish.is_some() {
        //     return finish.unwrap();
        // }
        if self.tree.as_ref().unwrap().root.borrow().pos_mov.borrow().pos != self.current_position {
            panic!("tree error");
        }
        // let node = if self.tree.as_mut().unwrap().root.borrow().childs.len() == 1 {
        //     self.tree.as_mut().unwrap().root.borrow().childs[0].clone()
        // } else {
        //     // Search in tree
        //     self.tree.as_mut().unwrap().search(self.mcts_lim)
        // };
        let finish = self.check_tree_for_finish();
        if finish.is_some() {
            return finish.unwrap()
        }
        let node = self.tree.as_mut().unwrap().search(self.mcts_lim);
        if apply {
            self.apply_node_move(node.clone());
        }
        let finish = self.check_tree_for_finish();
        if finish.is_some() {
            return finish.unwrap()
        }
        self.preparing_tree();

        let board_list =
            if apply {
                // to boards list
                let childs =
                    if !apply { node.clone().borrow().childs.clone() } else { self.tree.as_mut().unwrap().tree_childs() };
                // print!("to board list childs :{}\n", childs.len());

                let mut n_max =
                    childs.iter().max_by(|x, y|
                        x.borrow().N.cmp(&y.borrow().N)).unwrap().borrow().N;
                let mut w_min =
                    childs.iter().min_by(|x, y|
                        x.borrow().W.cmp(&y.borrow().W)).unwrap().borrow().W;
                let w_max =
                    childs.iter().max_by(|x, y|
                        x.borrow().W.cmp(&y.borrow().W)).unwrap().borrow().W;
                let mut delta_w = w_max - w_min;
                if delta_w == 0 {
                    delta_w = w_min;
                    w_min = w_min / 2;
                }
                if childs.len() == 1 { n_max *= 2; }
                let mut board_list: Vec<Vec<i32>> = vec![];
                for child in childs {
                    let mut board = vec![0 as i32; (self.position_environment.size * self.position_environment.size / 2) as usize];
                    for cell in &child.borrow().get_pos_mov().borrow().pos.cells {
                        if let Some(piece) = cell {
                            board[piece.pos] =
                                (if piece.is_king { 3 } else { 1 }) * if piece.color == Color::White { 1 } else { -1 }
                        }
                    }
                    board.push((child.borrow().W - w_min) as i32);
                    board.push(delta_w as i32);
                    board_list.push(board);
                }
                Some(board_list)
            } else { None };


        return MCTSRes { finish: None, board_list, pos_move: Some(node.borrow().pos_mov.clone()) };
    }

    #[wasm_bindgen]
    pub fn get_board_list_ts_n(&mut self) -> JsValue {
        return match serde_wasm_bindgen::to_value(&self.get_board_list()) {
            Ok(js) => js,
            Err(_err) => JsValue::UNDEFINED
        };
    }

    #[wasm_bindgen]
    pub fn mov_back(&mut self) {
        if self.position_history.borrow().list.len() > 1 {
            let mut pop = self.position_history.borrow_mut().pop();
            pop = self.position_history.borrow_mut().pop();
            self.current_position = pop.clone().unwrap().borrow().pos.clone();
            self.current_position.unmake_move(&pop.unwrap().borrow().mov.clone().unwrap());
            self.tree = None;
        }
    }

    #[wasm_bindgen]
    pub fn move_by_tree_index_ts_n(&mut self, i: usize) -> JsValue {
        return match serde_wasm_bindgen::to_value(&self.move_by_tree_index(i)) {
            Ok(js) => js,
            Err(_err) => JsValue::UNDEFINED
        };
    }

    pub fn move_by_tree_index(&mut self, i: usize) -> Option<FinishType> {
        let node = self.tree.as_mut().unwrap().tree_childs()[i].clone();
        self.tree.as_mut().unwrap().root = node.clone();
        return self.make_move_by_move_item(&node.borrow_mut().get_move().unwrap());
    }

    #[wasm_bindgen]
    pub fn move_by_index_ts_n(&mut self, i: i32) -> JsValue {
        if let Some(ref move_list) = self.current_position.get_move_list_cached().as_ref() {
            let len = move_list.list.len() as i32;
            if i >= 0 && i < len {
                self.make_move_by_move_item(&move_list.list[i as usize]);
                let finish = self.position_history.borrow_mut().finish_check();
                if finish.is_some() {
                    return match serde_wasm_bindgen::to_value(&finish.unwrap()) {
                        Ok(js) => js,
                        Err(_err) => JsValue::UNDEFINED
                    };
                }
                return {
                    JsValue::TRUE
                };
            } else {
                JsValue::FALSE
            }
        } else {
            return JsValue::FALSE;
        }
    }

    #[wasm_bindgen]
    pub fn get_best_move_rust(&mut self) -> BestPos {
        self.best_move(self.max_depth, i32::MIN / 2, i32::MAX / 2, 0, false)
    }

    pub fn state_(&self) -> String {
        return format!("{:?}", self.current_position.state);
    }

    #[wasm_bindgen]
    pub fn to_board(&self, pack_index: BoardPos) -> BoardPos {
        self.position_environment.pack_to_board[pack_index]
    }

    #[wasm_bindgen]
    pub fn to_pack(&self, board_index: BoardPos) -> BoardPos {
        self.position_environment.board_to_pack[board_index]
    }

    #[wasm_bindgen]
    pub fn get_move_list_for_front(&mut self) -> JsValue {
        let move_list = self.get_move_list(true);
        match serde_wasm_bindgen::to_value(&move_list) {
            Ok(js) => js,
            Err(_err) => JsValue::UNDEFINED,
        }
    }

    fn get_move_list(&mut self, for_front: bool) -> MoveList {
        self.current_position.get_move_list(for_front)
    }

    #[wasm_bindgen(getter = moveColor)]
    pub fn get_color(&self) -> JsValue {
        match self.current_position.next_move {
            Some(color) => match serde_wasm_bindgen::to_value(&color) {
                Ok(js) => js,
                Err(_err) => JsValue::UNDEFINED,
            },
            None => JsValue::UNDEFINED
        }
    }

    #[wasm_bindgen(setter = moveColor)]
    pub fn set_color(&mut self, color: Color) {
        self.current_position.next_move = Some(color);
    }

    #[wasm_bindgen]
    pub fn make_move_for_front(&mut self, pos_chain: &JsValue) -> Result<JsValue, JsValue> {
        let mut pos_list: Vec<BoardPos> = Vec::new();
        let iterator = js_sys::try_iter(pos_chain)?.ok_or_else(|| {
            "need to pass iterable JS values!"
        })?;
        for x in iterator {
            // If the iterators `next` method throws an error, propagate it
            // up to the caller.
            let x = x?;

            // If `x` is a number, add it to our array of numbers!
            if x.as_f64().is_some() {
                pos_list.push(x.as_f64().unwrap() as BoardPos);
            }
        }
        if !pos_list.is_empty() {
            if self.current_position.cells[pos_list[0] as usize].is_some() {
                let move_list = self.get_move_list(true);
                for mut move_item in move_list.list {
                    let mut i = 1;
                    let mut ok = true;
                    for mov in &move_item {
                        if pos_list.len() <= i {
                            ok = false;
                            break;
                        }
                        if pos_list[i] != mov.to() || pos_list[i - 1] != mov.from() {
                            ok = false;
                            break;
                        }
                        i += 1;
                    }
                    if ok && pos_list.len() == i {
                        self.current_position.make_move(&mut move_item);
                        let draw = self.position_history.borrow_mut().finish_check();
                        self.position_history.borrow_mut().push(
                            PositionAndMove::from(self.current_position.clone(), move_item));
                        return if draw.is_none() { Ok(JsValue::TRUE) } else {
                            Ok(serde_wasm_bindgen::to_value(&draw.unwrap()).unwrap())
                        };
                    }
                }
            }
        }
        Ok(JsValue::FALSE)
    }
}

#[cfg(test)]
mod tests {
    use std::io;
    use std::io::Write;
    use js_sys::Math::min;
    use crate::color::Color;
    use crate::game::Game;
    use crate::PositionHistory::FinishType::{BlackWin, Draw1, Draw2, Draw3, WhiteWin};
    use crate::piece::Piece;
    use crate::position_environment::PositionEnvironment;

    #[test]
    fn game_test() {
        let game = Game::new(8);
        assert!(game.current_position.state.kings_start_at.is_none());
    }

    #[test]
    fn game_quite_move() {
        let mut game = Game::new(8);
        game.current_position.next_move = Option::from(Color::White);
        game.insert_piece(Piece::new(13, Color::White, true));
        vec![2, 27, 24].iter().for_each(|pos| game.insert_piece(Piece::new(*pos, Color::White, false)));
        let list = game.get_move_list(true);
        print!("\ngame_quite_move {:?} \n", {
            let z: Vec<_> = list.list.iter().map(|x| x.mov.clone().unwrap()).collect();
            z
        });
        assert_eq!(list.list.len(), 15);
    }


    #[test]
    pub fn game_strike_list() {
        let mut game = Game::new(8);
        game.current_position.next_move = Some(Color::White);
        game.insert_piece(Piece::new(game.to_pack(47), Color::White, false));
        game.insert_piece(Piece::new(game.to_pack(63), Color::White, false));
        game.insert_piece(Piece::new(game.to_pack(15), Color::White, true));
        vec![54, 43, 20].iter()
            .for_each(|pos|
                game.insert_piece(Piece::new(game.to_pack(*pos), Color::Black, false)));
        for _t in 0..1000000 {
            let _list = game.get_move_list(true);
        }
        let list = game.get_move_list(true);
        print!("\ngame_quite_move {:?} \n", {
            let z: Vec<_> = list.list.iter().map(|x| x.strike.clone().unwrap()).collect();
            z
        });
        assert_eq!(list.list.len(), 5);
    }

    #[test]
    pub fn best_move() {
        let mut game = Game::new(8);
        game.current_position.next_move = Some(Color::White);
        game.insert_piece(Piece::new(game.to_pack(0), Color::White, true));
        vec![9, 11, 13, 25, 27, 29, 41, 43, 45].iter()
            .for_each(|pos|
                game.insert_piece(Piece::new(game.to_pack(*pos), Color::Black, false)));

        let best = &game.get_best_move_rust();
        assert_eq!(best.pos.as_ref().unwrap().borrow().pos.took_pieces.len(), 9);
        print!("\n best: {:?} \n", {
            best
        });
    }


    #[test]
    fn finish_cmp() {
        assert_eq!(Draw2, Draw1);
        assert_eq!(WhiteWin, WhiteWin);
        assert_eq!(BlackWin, BlackWin);
        assert_ne!(BlackWin, WhiteWin);
        assert_ne!(Draw2, WhiteWin);
        assert_eq!(WhiteWin > BlackWin, true);
        assert_eq!(BlackWin < WhiteWin, true);
        assert_eq!(WhiteWin < BlackWin, false);
        assert_eq!(WhiteWin > Draw3, true);
        assert_eq!(BlackWin < Draw1, true);
    }

    #[test]
    fn performance() {
        PositionEnvironment::game();
    }
}
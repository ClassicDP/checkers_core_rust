use std::cell::RefCell;
use std::io;
use std::io::Write;
use std::ops::Deref;
use std::rc::Rc;
use rand::{Rng, thread_rng};
use crate::color::Color;
use crate::color::Color::Black;
use crate::game::Game;
use crate::mcts::{McTree, Node};
use crate::piece::Piece;

include!("lib.rs");

#[derive(Debug)]
struct MoveAsStrike {
    from: usize,
    to: usize,
    take: usize,
}

#[derive(Debug)]
struct MoveAsQuite {
    from: usize,
    to: usize,
}

pub fn random_game_test() {
    let v_w: Vec<_> = vec![0; 12].iter().enumerate()
        .map(|(i, x)| if i / 4 % 2 == 0 { 2 * i } else { 2 * i + 1 }).collect();
    let v_b: Vec<_> = vec![0; 12].iter().enumerate()
        .map(|(i, x)| 63 - if i / 4 % 2 == 0 { 2 * i } else { 2 * i + 1 }).collect();
    let mut game_count = 0;
    loop {
        let mut game = Game::new(8);
        game.current_position.next_move = Some(Color::White);
        v_w.iter()
            .for_each(|pos|
                game.insert_piece(Piece::new(game.to_pack(*pos), Color::White, false)));
        v_b.iter()
            .for_each(|pos|
                game.insert_piece(Piece::new(game.to_pack(*pos), Color::Black, false)));
        while game.position_history.borrow_mut().finish_check().is_none() {
            // print!("state {}\n", game.state_());
            // print!("history {:?}\n", game.position_history.len());
            if game.position_history.borrow().len() % 2 == 10 {
                let moves_list = game.current_position.get_move_list_cached();
                let i = thread_rng().gen_range(0..moves_list.as_ref().as_ref().unwrap().list.len());
                game.move_by_index_ts_n(i as i32);
                // let ref mut random_move = moves_list.borrow_mut().list[i];
                // game.make_move_by_move_item(random_move);
            } else {
                let ref mut best_pos = game.get_best_move_rust();
                game.make_move_by_pos_item(best_pos);
            }
        }
        print!("end: {} {} {:?}\n", game_count, game.position_history.borrow().len(), game.position_history.borrow_mut().finish_check());
        print!("state {}\n", game.state_());
        game_count += 1;
    }
}

pub fn best_move_triangle() {
    let mut game = Game::new(8);
    game.set_depth(4);
    // game.current_position.next_move = Some(Color::Black);
    // vec![31].iter()
    //     .for_each(|pos|
    //         game.insert_piece(Piece::new(game.to_pack(*pos), Color::White, true)));
    // vec![43, 36, 20].iter()
    //     .for_each(|pos|
    //         game.insert_piece(Piece::new(game.to_pack(*pos), Color::Black, true)));

    vec![0, 2, 4, 6, 9, 11, 13, 15, 16, 18, 20, 22].iter()
        .for_each(|pos|
            game.insert_piece(Piece::new(game.to_pack(*pos), Color::White, false)));
    vec![0, 2, 4, 6, 9, 11, 13, 15, 16, 18, 20, 22].iter().map(|x| 63 - x).collect::<Vec<_>>().iter()
        .for_each(|pos|
            game.insert_piece(Piece::new(game.to_pack(*pos), Color::Black, false)));
    game.current_position.next_move = Option::from(Color::White);
    // game.current_position.next_move = Some(Color::Black);
    // vec![52, 54, 38, 63].iter()
    //     .for_each(|pos|
    //         game.insert_piece(Piece::new(game.to_pack(*pos), Color::Black, false)));
    // vec![0, 9].iter()
    //     .for_each(|pos|
    //         game.insert_piece(Piece::new(game.to_pack(*pos), Color::White, true)));


    // game.position_history.borrow_mut().push(PositionAndMove::from_pos(game.current_position));
    game.tree = Some(McTree::new(game.current_position.clone(), game.position_history.clone()));


    let mut node: Option<Rc<RefCell<Node>>> = None;
    loop {
        let mut new_root_found = false;
        if let Some(node) = node.clone() {
            if let Some(new_root_node) =
                node.borrow()
                    .childs.iter()
                    .find(|x| x.borrow().get_pos_mov().borrow().pos == game.current_position) {
                game.tree = Some(McTree::new_from_node(new_root_node.clone(), game.position_history.clone()));
                new_root_found = true;
            }
        }
        if !new_root_found {
            game.tree = Some(McTree::new(game.current_position.clone(), game.position_history.clone()));
        }
        if let Some(ref mut tree) = game.tree {
            node = tree.search(200_000);
            if node.is_none() {
                print!("deeps win");
                break;
            }
            print!("___ {:?} \n", game.tree.as_ref().unwrap().root_map());
            let mov = &node.clone().unwrap().borrow().get_move().unwrap().clone();
            // print!("{:?}\n", &mov);
            // io::stdout().flush().unwrap();
            game.make_move_by_move_item(mov);
            if game.position_history.borrow_mut().finish_check().is_some() {
                print!("move queue by deeps, but {:?} \n", game.position_history.borrow_mut().finish_check());
                break;
            }
            let bp = game.get_best_move_rust();
            game.make_move_by_move_item(&bp.get_move_item());
            if game.position_history.borrow_mut().finish_check().is_some() {
                print!("move queue by mcts, but {:?} \n", game.position_history.borrow_mut().finish_check());
                break;
            }
        }
    }
    print!("{:?}\n", &game.position_history.borrow_mut().last().borrow().pos.state);

    return;
    let mut game = Game::new(8);
    game.set_depth(8);
    game.current_position.next_move = Some(Color::Black);
    vec![31].iter()
        .for_each(|pos|
            game.insert_piece(Piece::new(game.to_pack(*pos), Color::White, true)));
    vec![43, 36, 20].iter()
        .for_each(|pos|
            game.insert_piece(Piece::new(game.to_pack(*pos), Color::Black, true)));

    use crate::moves::PieceMove;
    while game.position_history.borrow_mut().finish_check().is_none() {
        print!("state {}\n", game.state_());
        print!("history {:?}\n", game.position_history.borrow().len());
        let best = game.get_best_move_rust();
        print!("{}", {
            if best.get_move_item().strike.is_some() {
                format!("move: {:?}\n", best.get_move_item().strike.unwrap().vec.iter().map(
                    |x|
                        MoveAsStrike {
                            from: game.to_board(x.from()),
                            to: game.to_board(x.to()),
                            take: game.to_board(x.take().unwrap()),
                        }).collect::<Vec<_>>())
            } else {
                format!("move: {:?}\n", [best.get_move_item().mov.unwrap()].iter().map(|x|
                    MoveAsQuite {
                        from: game.to_board(x.from()),
                        to: game.to_board(x.to),
                    }).collect::<Vec<_>>())
            }
        });
        game.make_move_by_pos_item(&best);
    }
    print!("{:?}", game.position_history.borrow_mut().finish_check());
}

pub fn main() {
    loop {
        best_move_triangle();
    }
    return;
    // random_game_test();
    let mut game = Game::new(8);
    game.insert_piece(Piece::new(22, Color::White, false));
    game.insert_piece(Piece::new(4, Color::Black, true));
    game.insert_piece(Piece::new(21, Color::Black, true));
    game.insert_piece(Piece::new(20, Color::Black, true));
    game.insert_piece(Piece::new(12, Color::Black, true));
    game.insert_piece(Piece::new(13, Color::Black, true));
    game.insert_piece(Piece::new(26, Color::Black, true));
    game.current_position.next_move = Some(Color::White);
    let now = Instant::now();
    for _i in 0..1000000 {
        let mut list = game.current_position.get_move_list(false);
        let mut pos_list: Vec<_> = {
            list.list.iter_mut().map(|x| {
                let mut pos = game.current_position.make_move_and_get_position(x);
                game.current_position.unmake_move(x);
                pos.pos.evaluate();
                pos
            }).collect()
        };
        pos_list.sort_by_key(|x|
            x.pos.eval.unwrap() * if x.pos.next_move.unwrap() == Color::White { -1 } else { 1 });
        let po = game.current_position.make_move_and_get_position(&mut list.list[0]);
        game.position_history.borrow_mut().finish_check();
        if po.pos != po.pos { break; }
        game.current_position.unmake_move(&mut list.list[0]);
    }
    print!("strike:  {:.2?}\n", now.elapsed());

    let mut game = Game::new(8);
    game.insert_piece(Piece::new(game.to_pack(47), Color::White, false));
    game.insert_piece(Piece::new(game.to_pack(63), Color::White, false));
    game.insert_piece(Piece::new(game.to_pack(15), Color::White, true));
    vec![54, 43, 20].iter()
        .for_each(|pos|
            game.insert_piece(Piece::new(game.to_pack(*pos), Color::Black, false)));
    game.current_position.next_move = Some(Color::White);
    use std::time::Instant;
    let now = Instant::now();
    for _i in 0..1000000 {
        let mut list = game.current_position.get_move_list(false);
        let po = game.current_position.make_move_and_get_position(&mut list.list[0]);
        if po.pos != po.pos { break; }
        game.current_position.unmake_move(&mut list.list[0]);
    }
    print!("strike 2:  {:.2?}\n", now.elapsed());


    let mut game = Game::new(8);
    game.insert_piece(Piece::new(game.to_pack(16), Color::White, false));
    game.insert_piece(Piece::new(game.to_pack(18), Color::White, false));
    game.insert_piece(Piece::new(game.to_pack(20), Color::White, false));
    game.insert_piece(Piece::new(game.to_pack(22), Color::White, false));
    game.current_position.next_move = Some(Color::White);

    let now = Instant::now();
    for _i in 0..1000000 {
        let mut list = game.current_position.get_move_list(false);
        let po = game.current_position.make_move_and_get_position(&mut list.list[0]);
        if po.pos != po.pos { break; }
        game.current_position.unmake_move(&mut list.list[0]);
    }
    print!("simple: {:.2?}\n", now.elapsed());
}
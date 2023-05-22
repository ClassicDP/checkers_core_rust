
use std::collections::HashMap;
use std::io;
use std::io::Write;
use std::sync::{Arc, Mutex, RwLock};
use rand::{Rng, thread_rng};
use rayon::prelude::IntoParallelRefIterator;
use crate::cache_map::CacheMap;
use crate::color::Color;
use crate::game::Game;
use crate::mcts::{Cache, CacheItem, Node, PositionWN};
use crate::piece::Piece;
use rayon::prelude::*;
use std::iter::Iterator;
use tokio::runtime::Runtime;
use crate::cache_db::CacheDb;
use crate::PositionHistory::FinishType;
use crate::PositionHistory::FinishType::{BlackWin, WhiteWin};
include!("lib.rs");
#[derive(Debug)]
pub struct Score {
    m: i32,
    d: i32
}
pub type ThreadScore = Arc<Mutex<Score>>;

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

pub fn init(game: &mut Game) {
    *game = Game::new(8);
    vec![0, 2, 4, 6, 9, 11, 13, 15, 16, 18, 20, 22].iter()
        .for_each(|pos|
            game.insert_piece(Piece::new(game.to_pack(*pos), Color::White, false)));
    vec![0, 2, 4, 6, 9, 11, 13, 15, 16, 18, 20, 22].iter().map(|x| 63 - x).collect::<Vec<_>>().iter()
        .for_each(|pos|
            game.insert_piece(Piece::new(game.to_pack(*pos), Color::Black, false)));
    game.current_position.next_move = Option::from(Color::White);
    // vec![4].iter()
    //     .for_each(|pos|
    //         game.insert_piece(Piece::new(game.to_pack(*pos), Color::Black, true)));
    // vec![0, 29, 34].iter()
    //     .for_each(|pos|
    //         game.insert_piece(Piece::new(game.to_pack(*pos), Color::White, true)));
    // game.current_position.next_move = Option::from(Color::White);
}

pub fn init_test(game: &mut Game) {
    *game = Game::new(8);
    vec![43].iter()
        .for_each(|pos|
            game.insert_piece(Piece::new(game.to_pack(*pos), Color::White, false)));
    vec![50].iter()
        .for_each(|pos|
            game.insert_piece(Piece::new(game.to_pack(*pos), Color::Black, false)));
    game.current_position.next_move = Option::from(Color::Black);
    // vec![4].iter()
    //     .for_each(|pos|
    //         game.insert_piece(Piece::new(game.to_pack(*pos), Color::Black, true)));
    // vec![0, 29, 34].iter()
    //     .for_each(|pos|
    //         game.insert_piece(Piece::new(game.to_pack(*pos), Color::White, true)));
    // game.current_position.next_move = Option::from(Color::White);
}

pub fn deep_mcts(mut cache: Cache, passes: i32, score: ThreadScore) {
    let mut game = Game::new(8);
    let score_calc = |finish: &FinishType, neuron_start: bool| {
        if (*finish == BlackWin && !neuron_start) ||
            (*finish == WhiteWin) && neuron_start {
            score.lock().unwrap().m +=1;
        };
        if (*finish == WhiteWin && !neuron_start) ||
            (*finish == BlackWin) && neuron_start {
            score.lock().unwrap().d +=1;
        };
    };
    loop {
        init(&mut game);
        game.init_tree();
        if game.tree.is_some() {
            game.tree.as_mut().unwrap().set_cache(cache);
        }
        let neuron_start = thread_rng().gen_range(0.0..2.0) > 1.0;
        if neuron_start {
            game.set_mcts_lim(passes);
            game.find_mcts_and_make_best_move(true);
        }
        loop {
            let finish = game.position_history.borrow_mut().finish_check();
            if let Some(finish) = finish {
                score_calc(&finish, neuron_start);
                print!("mcts start: {:?} {:?}  {:?} {:?}\n",
                       neuron_start, finish, game.position_history.borrow().list.len(), score.lock().unwrap());
                break;
            }
            game.set_depth(3);
            let best_move = game.get_best_move_rust();
            // print!("{:?}\n", best_move.get_move_item());
            io::stdout().flush().unwrap();
            game.make_best_move(&best_move);
            let finish = game.position_history.borrow_mut().finish_check();
            if let Some(finish) = finish {
                score_calc(&finish, neuron_start);
                print!("mcts start: {:?} {:?}  {:?} {:?}\n",
                       neuron_start, finish, game.position_history.borrow().list.len(), score.lock().unwrap());
                break;
            };
            game.set_mcts_lim(passes);
            game.find_mcts_and_make_best_move(true);

            // println!("_");
            // game.set_depth(5);
            // game.set_mcts_lim(300000);
            // game.mix_method(true);
            // print!("{:?}\n", mov.pos_move.unwrap().borrow().mov);
            // print!("{:?}\n", game.tree.as_ref().unwrap().cache.lock().unwrap().freq_list.data_size);
        }

        cache = game.tree.as_mut().unwrap().cache.clone();
        game.tree = None;

    }
}

pub async fn mcts() {
    let mut game = Game::new(8);

    init(&mut game);
    loop {
        let next = game.find_mcts_and_make_best_move(true).await;
        if next.board_list.is_some() {
            let list0 = next.board_list.unwrap();
            let mut list = list0.clone();
            let i = list[0].len() - 2;
            list.sort_by(|x, y| x[i].cmp(&y[i]));
            let i = list.len() - 1;
            // if i > 0 { i -= 1; }
            let x0 = list[i].clone();
            let index = list0.iter().enumerate().find(|x| *x.1 == x0).unwrap().0;
            let finish = game.move_by_tree_index(index);
            if finish.is_some() {
                print!("{:?}  {:?}\n", finish, game.position_history.borrow().list.len());
                io::stdout().flush().unwrap();
                init(&mut game);
            }
        } else {
            print!("{:?}  {:?}\n", next.finish, game.position_history.borrow().list.len());
            io::stdout().flush().unwrap();
            init(&mut game);
        }
    }
}


pub async fn mcts_test() {
    let mut game = Game::new(8);

    init_test(&mut game);
    loop {
        let next = game.find_mcts_and_make_best_move(true).await;
        if next.board_list.is_some() {
            let list0 = next.board_list.unwrap();
            let mut list = list0.clone();
            let i = list[0].len() - 2;
            list.sort_by(|x, y| x[i].cmp(&y[i]));
            let mut i = list.len() - 1;
            // if i > 0 { i -= 1; }
            let x0 = list[i].clone();
            let index = list0.iter().enumerate().find(|x| *x.1 == x0).unwrap().0;
            let finish = game.move_by_tree_index(index);
            if finish.is_some() {
                print!("{:?}  {:?}\n", finish, game.position_history.borrow().list.len());
                io::stdout().flush().unwrap();
                init(&mut game);
            }
        } else {
            print!("{:?}  {:?}\n", next.finish, game.position_history.borrow().list.len());
            io::stdout().flush().unwrap();
            init(&mut game);
        }
    }
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
        print!("end: {} {} {:?}\n", game_count, game.position_history.borrow().len(),
               game.position_history.borrow_mut().finish_check());
        print!("state {}\n", game.state_());
        game_count += 1;
    }
}


pub fn main() {
    let arg = std::env::args().collect::<Vec<_>>();
    let mut threads_q: usize = 1;
    let mut cache_size: usize = 10_000_000;
    let mut pass_q: usize = 5_000;
    println!("{:?}", arg);
    let score: ThreadScore = Arc::new(Mutex::new(Score {d: 0, m: 0}));
    let pos = arg.iter().position(|x|*x=="+++".to_string());
    if pos.is_some() && arg.len() - pos.unwrap() ==4{

        [threads_q, cache_size, pass_q] = <[usize; 3]>::try_from(
            arg[pos.unwrap()+1..].iter().map(|x| x.parse().unwrap()).collect::<Vec<_>>()).unwrap();
        println!("set threads_q: {},  cache_size: {}, pass_q: {}", threads_q, cache_size, pass_q);
    }
    let runtime = Runtime::new().unwrap();
    runtime.block_on(async {
        let cache = Cache(Arc::new(RwLock::new(Some(CacheDb::new(
            CacheItem::key, "checkers".to_string(),
            "nodes".to_string(), cache_size as u64 ,
            1, 1).await))));
        Arc::new(vec![0;threads_q]).par_iter().for_each(|_|
            deep_mcts(cache.clone(), pass_q as i32, score.clone()));
    });


}
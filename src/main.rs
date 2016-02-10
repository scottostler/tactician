mod cards;
mod deciders;
mod game;
mod tree_search;
mod search_decider;
mod util;
mod nim;

#[macro_use] extern crate itertools;
#[macro_use] extern crate lazy_static;
extern crate getopts;
extern crate rand;
extern crate core;


fn run_games(num_games: i32, players: &mut Vec<Box<game::Decider>>, debug: bool) {
    println!("Running {} game(s)...", num_games);

    let mut results = vec![0.0; 2];
    for _ in 0..num_games {
        let r = game::run_game(players, debug);
        for (i, score) in r.iter().enumerate() {
            results[i] += *score;
        }
    }

    println!("");
    for (i, score) in results.iter().enumerate() {
        println!("Player {} won {} game(s)", players[i].description(), score);
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let opts = getopts::Options::new();
    
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => { m }
        Err(f) => { panic!(f.to_string()) }
    };
    
    let num_games: i32 = match matches.free.first() {
        Some(s) => s.parse::<i32>().unwrap(),
        None    => 1
    };

    let debug = true;
    let ctx = game::EvalContext { debug: false, rng: util::randomly_seeded_weak_rng() };

    let mut players = vec![
        Box::new(search_decider::SearchDecider { ctx: ctx, debug: debug, iterations: 10000 }) as Box<game::Decider>,
        Box::new(deciders::BigMoney) as Box<game::Decider>];
    
    run_games(num_games, &mut players, debug);
}

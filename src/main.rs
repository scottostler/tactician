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

fn player_for_string(s: String, debug: bool) -> Box<game::Decider> {
    let s = s.to_lowercase();
    
    match s.to_lowercase().as_ref() {
        "bigmoney"  => Box::new(deciders::BigMoney),
        "tactician" => {
            let simulator_ctx = game::EvalContext { debug: false, rng: util::randomly_seeded_weak_rng() };
            Box::new(search_decider::SearchDecider { ctx: simulator_ctx, debug: debug, iterations: 10000 })
        },
        "random"    => Box::new(deciders::RandomDecider::new()),
        _           => panic!("Unknown player {}", s)
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut opts = getopts::Options::new();
    opts.optflag("d", "debug", "enable debug logging");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => { m }
        Err(f) => { panic!(f.to_string()) }
    };
    
    let num_games: i32 = match matches.free.first() {
        Some(s) => s.parse::<i32>().unwrap(),
        None    => 1
    };

    let debug = matches.opt_present("debug");
    
    let first_player = player_for_string(
        matches.free.get(1).unwrap_or(&String::from("tactician")).clone(), debug);
    let second_player = player_for_string(
        matches.free.get(2).unwrap_or(&String::from("bigmoney")).clone(), debug);
    
    let mut players = vec![first_player, second_player];
    run_games(num_games, &mut players, debug);
}

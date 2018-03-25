mod cards;
mod deciders;
mod game;
mod game_scoring;
mod game_logging;
mod tree_search;
mod search_decider;
mod util;
mod nim;

extern crate core;
extern crate getopts;
extern crate itertools;
#[macro_use]
extern crate lazy_static;
extern crate rand;

fn run_games(num_games: u32, players: &mut Vec<Box<game::Decider>>, debug: bool) {
    if num_games > 1 {
        println!("Running {} game(s)", num_games);
    }

    let mut results = vec![0.0; 2];
    for i in 0..num_games {
        if num_games > 1 {
            let title = format!("Game {}", i + 1);
            println!("");
            println!("========================================");
            println!("|{: ^38}|", title);
            println!("========================================");
            println!("");
        }
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
    match s.to_lowercase().as_ref() {
        "bigmoney" => Box::new(deciders::BigMoney),
        "tactician" => {
            let num_iters = 10000;
            let simulator_ctx = game::EvalContext {
                debug: false,
                rng: util::randomly_seeded_weak_rng(),
            };
            Box::new(search_decider::SearchDecider {
                ctx: simulator_ctx,
                debug: debug,
                iterations: num_iters,
            })
        }
        "random" => Box::new(deciders::RandomDecider::new()),
        _ => panic!("Unknown player {}", s),
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut opts = getopts::Options::new();
    opts.optflag("d", "debug", "enable debug logging");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => panic!(f.to_string()),
    };

    let num_games = match matches.free.first() {
        Some(s) => s.parse::<u32>().unwrap(),
        None => 1,
    };

    if num_games == 0 {
        println!("I can't play zero games. That’s silly!");
        std::process::exit(1);
    }

    let debug = matches.opt_present("debug");

    let first_player = player_for_string(
        matches
            .free
            .get(1)
            .unwrap_or(&String::from("tactician"))
            .clone(),
        debug,
    );
    let second_player = player_for_string(
        matches
            .free
            .get(2)
            .unwrap_or(&String::from("bigmoney"))
            .clone(),
        debug,
    );

    let mut players = vec![first_player, second_player];
    run_games(num_games, &mut players, debug);
}

mod cards;
mod deciders;
mod game;
mod util;

extern crate getopts;
#[macro_use]
extern crate lazy_static;
extern crate rand;

fn main() {
    let num_games = 1000;
    println!("Running {} games...", num_games);

    let players = vec![
        Box::new(deciders::RandomDecider) as Box<game::Decider>,
        Box::new(deciders::BigMoney) as Box<game::Decider>];

    let mut results = vec![0.0; 2];

    for _ in 0..num_games {
        let r = game::run_game(&players, false);
        for (i, score) in r.iter().enumerate() {
            results[i] += *score;
        }
    }
    println!("");
    for (i, score) in results.iter().enumerate() {
        println!("Player {} won {} game(s)", players[i].description(), score);
    }
}


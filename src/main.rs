#![allow(dead_code)]

#[macro_use]
extern crate lazy_static;

extern crate getopts;
extern crate rand;

use rand::{thread_rng, Rng, ThreadRng, sample};
use std::collections::HashMap;
use std::fmt;

const EMPTY_PILES_FOR_GAME_END: i32 = 3;
const PLAYER_HAND_SIZE: usize = 5;
const VP_PILE_SIZE: i32 = 8;
const CURSE_PILE_SIZE: i32 = 10;

#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
struct PlayerIdentifier(pub i32);

#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
struct CardIdentifier(pub i32);

impl fmt::Display for CardIdentifier {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", lookup_card(self).name)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum Phase {
    StartTurn,
    Action,
    BuyPlayTreasure,
    BuyPurchaseCard,
    Cleanup,
    EndTurn
}

struct Card {
    identifier: CardIdentifier,
    name: &'static str,
    cost: i32,
    coin_value: i32,
    vp_value: i32,
}

#[derive(Clone)]
struct Player {
    name: String,
    hand: Vec<CardIdentifier>,
    discard: Vec<CardIdentifier>,
    deck: Vec<CardIdentifier>,
}

const COPPER   : Card = Card { identifier: CardIdentifier(1), name: "Copper", cost: 0, coin_value: 1, vp_value: 0 };
const SILVER   : Card = Card { identifier: CardIdentifier(2), name: "Silver", cost: 3, coin_value: 2, vp_value: 0 };
const GOLD     : Card = Card { identifier: CardIdentifier(3), name: "Gold", cost: 6, coin_value: 3, vp_value: 0 };
const ESTATE   : Card = Card { identifier: CardIdentifier(4), name: "Estate", cost: 2, coin_value: 0, vp_value: 1 };
const DUCHY    : Card = Card { identifier: CardIdentifier(5), name: "Duchy", cost: 5, coin_value: 0, vp_value: 3 };
const PROVINCE : Card = Card { identifier: CardIdentifier(6), name: "Province", cost: 8, coin_value: 0, vp_value: 6 };
const CURSE    : Card = Card { identifier: CardIdentifier(7), name: "Curse", cost: 0, coin_value: 0, vp_value: -1 };

lazy_static! {
    static ref CARDS : Vec<Card> = vec![COPPER, SILVER, GOLD, ESTATE, DUCHY, PROVINCE, CURSE];
}

fn lookup_card(ci: &CardIdentifier) -> &Card {
    return &CARDS[(ci.0 - 1) as usize];
}

fn card_names(identifiers: &Vec<CardIdentifier>) -> String {
    return identifiers.iter()
        .map(|ci| lookup_card(ci).name.to_string())
        .collect::<Vec<String>>().join(", ");
}

fn score_cards(identifiers: &Vec<CardIdentifier>) -> i32 {
    return identifiers.iter()
        .map(|ci| lookup_card(ci).vp_value)
        .fold(0, |sum, i| sum + i);
}

fn log(s:String) {
    println!("{}", s);
}

fn subtract_vector<T: fmt::Display + Eq>(vs: &mut Vec<T>, s: &Vec<T>) {
    for x in s.iter() {
        let idx = match vs.iter().position(|v| *v == *x) {
            Some(idx) => idx,
            None => panic!("Unable to find index for {}", x)
        };
        vs.remove(idx);
    }
}

impl Card {
    fn is_treasure(&self) -> bool {
        return self.coin_value > 0;
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum DecisionType {
    PlayTreasures,
    BuyCard
}

#[derive(Clone)]
struct Decision {
    player: PlayerIdentifier,
    decision_type: DecisionType,
    choices: Vec<CardIdentifier>,
    range: (usize, usize),
}

trait Decider {
    fn description(&self) -> String;
    fn make_decision(&self, d: &Decision, g: &Game) -> Vec<CardIdentifier>;
}

struct BigMoney;

impl Decider for BigMoney {

    fn description(&self) -> String { return "Big Money".into(); }

    fn make_decision(&self, d: &Decision, g: &Game) -> Vec<CardIdentifier> {
        match d.decision_type {
            DecisionType::PlayTreasures => return d.choices.clone(),
            DecisionType::BuyCard => {
                let cs = g.coins;
                if cs >= PROVINCE.cost {
                    return vec![PROVINCE.identifier];
                } else if cs >= GOLD.cost {
                    return vec![GOLD.identifier];
                } else if cs >= SILVER.cost {
                    return vec![SILVER.identifier];
                } else {
                    return vec![];
                }
            }
        }
    }
}

struct RandomDecider;

impl Decider for RandomDecider {

    fn description(&self) -> String { return "Random".into(); }

    fn make_decision(&self, d: &Decision, _: &Game) -> Vec<CardIdentifier> {
        if d.decision_type == DecisionType::PlayTreasures {
            return d.choices.clone();
        }

        let mut rng = thread_rng();
        let n = match d.range.0 == d.range.1 {
            true => d.range.0,
            false => rng.gen_range(d.range.0, d.range.1 + 1) as usize,
        };
        return sample(&mut rng, d.choices.clone(), n);
    }
}

impl Player {
    fn draw_cards(&mut self, n:usize, ctx: &mut EvalContext) {
        let mut drawn = match self.deck.len() >= n {
            true => {
                let pivot = self.deck.len() - n;
                self.deck.split_off(pivot)
            }
            false => {
                let mut first_draw: Vec<CardIdentifier> = self.deck.clone();

                ctx.rng.shuffle(&mut self.discard);
                self.deck = self.discard.clone();
                self.discard.clear();

                if ctx.debug {
                    println!("{} shuffled", self.name);
                }

                let second_n = std::cmp::min(self.deck.len(), n - first_draw.len());
                let pivot = self.deck.len() - second_n;
                let mut second_draw = self.deck.split_off(pivot);
                first_draw.append(&mut second_draw);
                first_draw
            }
        };

        if ctx.debug {
            println!("{} drew {} cards", self.name, drawn.len());
        }

        self.hand.append(&mut drawn);
    }

    fn discard_hand(&mut self, ctx: &mut EvalContext) {
        if ctx.debug {
            println!("{} discarded {}", self.name, card_names(&self.hand));
        }

        self.discard.extend(&self.hand);
        self.hand.clear();
    }

    fn all_cards(&self) -> Vec<CardIdentifier> {
        let mut ret = Vec::new();
        ret.extend(&self.hand);
        ret.extend(&self.deck);
        ret.extend(&self.discard);
        return ret;
    }
}

#[derive(Clone)]
struct Game {
    turn: i32,
    active_player: PlayerIdentifier,
    phase: Phase,
    actions: i32,
    buys: i32,
    coins: i32,
    piles: HashMap<CardIdentifier, i32>,
    play_area: Vec<CardIdentifier>,
    players: Vec<Player>,
    pending_decision: Option<Decision>
}

struct EvalContext {
    rng: ThreadRng,
    debug: bool
}

impl Game {
    fn initialize_game(&mut self, ctx: &mut EvalContext) {
        if ctx.debug {
            println!("The game is afoot!");
        }
        for mut p in self.players.iter_mut() {
            p.draw_cards(PLAYER_HAND_SIZE, ctx);
        }
    }

    fn is_game_over(&self) -> bool {
        if self.phase != Phase::EndTurn {
            return false;
        } else if self.piles[&PROVINCE.identifier] == 0 {
            return true;
        } else {
            let mut n = 0;
            for count in self.piles.values() {
                if *count == 0 {
                    n += 1;
                }

                if n >= EMPTY_PILES_FOR_GAME_END {
                    return true;
                }
            }
            return false;
        }
    }

    fn next_turn(&mut self) {
        if self.active_player.0 + 1 == self.players.len() as i32 {
            self.turn += 1;
            self.active_player = PlayerIdentifier(0);
        } else {
            self.active_player = PlayerIdentifier(self.active_player.0 + 1);
        }

        self.phase = Phase::StartTurn;
        self.actions = 1;
        self.buys = 1;
        self.coins = 0;
    }

    fn advance_game(&mut self, ctx: &mut EvalContext) {
        match self.phase {
            Phase::StartTurn => {
                if ctx.debug {
                    let ref player = self.players[self.active_player.0 as usize];
                    println!("----- Turn {}, {} -----", self.turn, player.name);
                }
                self.phase = Phase::Action;
            }
            Phase::Action => {
                // TODO: actually implement actions
                self.phase = Phase::BuyPlayTreasure;
            }
            Phase::BuyPlayTreasure => {
                let treasures = self.players[self.active_player.0 as usize].hand
                    .iter().cloned().filter(|c| lookup_card(c).is_treasure())
                    .collect::<Vec<CardIdentifier>>();

                let treasure_len = treasures.len();
                self.pending_decision = Some(Decision {
                    player: self.active_player,
                    decision_type: DecisionType::PlayTreasures,
                    choices: treasures,
                    range: (0, treasure_len)
                })
            }
            Phase::BuyPurchaseCard => {
                if self.buys == 0 {
                    self.phase = Phase::Cleanup;
                } else {
                    let available_coins = self.coins;
                    let mut buyable = Vec::new();
                    for (ci, &num) in self.piles.iter() {
                        if num > 0 && lookup_card(ci).cost <= available_coins {
                            buyable.push(*ci);
                        }
                    }

                    self.pending_decision = Some(Decision {
                        player: self.active_player,
                        decision_type: DecisionType::BuyCard,
                        choices: buyable,
                        range: (0, 1)
                    })
                }
            }
            Phase::Cleanup => {
                let ref mut player = self.players[self.active_player.0 as usize];
                player.discard_hand(ctx);
                player.discard.extend(&self.play_area);
                self.play_area.clear();
                player.draw_cards(PLAYER_HAND_SIZE, ctx);
                self.phase = Phase::EndTurn;
            }
            Phase::EndTurn => {
                self.next_turn();
            }
        }
    }

    fn resolve_decision(&mut self, decision: &Decision, result: Vec<CardIdentifier>, ctx: &mut EvalContext) {
        match decision.decision_type {
            DecisionType::BuyCard => {
                match result.first() {
                    Some(ci) => {
                        let c = lookup_card(ci);
                        assert!(self.buys > 0, "Must have a buy");
                        assert!(self.coins >= c.cost, "Must have enough coins");
                        assert!(self.piles[ci] > 0, "Pile must not be empty");
                        self.buys -= 1;
                        self.coins -= c.cost;
                        match self.piles.get_mut(ci) {
                            Some(l) => *l -= 1,
                            None => panic!("Cannot find pile for {}", c.name),
                        }
                        self.players[decision.player.0 as usize].discard.push(*ci);

                        if ctx.debug {
                            println!("{} buys {}", self.players[decision.player.0 as usize].name, c.name);
                        }
                    }
                    None => self.phase = Phase::Cleanup
                }
            }
            DecisionType::PlayTreasures => {
                if result.len() > 0 {
                    let cards = result.iter().map(|ci| lookup_card(ci) ).collect::<Vec<&Card>>();

                    for c in &cards {
                        assert!(c.is_treasure(), "Can only play treasures");
                        self.coins += c.coin_value;
                    }

                    let ref mut player = self.players[decision.player.0 as usize];

                    if ctx.debug {
                        let names = cards.iter().map(|c| c.name.into() ).collect::<Vec<String>>();
                        println!("{} plays {}", player.name, names.join(", "));
                    }

                    self.play_area.extend(&result);
                    subtract_vector::<CardIdentifier>(&mut player.hand, &result);
                }
                self.phase = Phase::BuyPurchaseCard;
            }
        }
    }

}

fn fresh_player(name: &String) -> Player {
    let mut discard = std::iter::repeat(COPPER.identifier).take(7).collect::<Vec<CardIdentifier>>();
    discard.extend(std::iter::repeat(ESTATE.identifier).take(3));
    return Player { name: name.clone(), hand: Vec::new(), deck: Vec::new(), discard: discard };
}

fn fresh_game(player_names: &Vec<String>) -> Game {
    let players = player_names.iter().map(|name| {
            return fresh_player(name);
        }).collect();

    let piles: HashMap<CardIdentifier, i32> =  vec![
        (PROVINCE.identifier, VP_PILE_SIZE),
        (DUCHY.identifier, VP_PILE_SIZE),
        (ESTATE.identifier, VP_PILE_SIZE),
        (GOLD.identifier, 30),
        (SILVER.identifier, 40),
        (COPPER.identifier, 46),
        (CURSE.identifier, CURSE_PILE_SIZE)
    ].into_iter().collect();

    return Game {
        turn: 1,
        active_player: PlayerIdentifier(0),
        phase: Phase::StartTurn,
        actions: 1,
        buys: 1,
        coins: 0,
        piles: piles,
        play_area: Vec::new(),
        players: players,
        pending_decision: None,
    };
}

fn run_game(players: &Vec<Box<Decider>>, debug: bool) -> Vec<f32> {
    let mut ctx = EvalContext { rng: thread_rng(), debug: debug };

    let player_names = players.iter().map(|d| d.description()).collect::<Vec<_>>();
    let mut game = fresh_game(&player_names);
    game.initialize_game(&mut ctx);

    while !game.is_game_over() {
        let decision: Option<Decision> = std::mem::replace(&mut game.pending_decision, None);
        match decision {
            Some(ref d) => {
                let choice = players[d.player.0 as usize].make_decision(d, &game);
                game.resolve_decision(d, choice, &mut ctx);
            }
            None => game.advance_game(&mut ctx)
        }
    }

    let turn_count = game.turn;
    let active_player = game.active_player;
    let points = game.players.iter().enumerate().map(|(i, p)| {
        let score = score_cards(&p.all_cards());
        if i <= (active_player.0 as usize) {
            (score, turn_count)
        } else {
            (score, turn_count - 1)
        }
    }).collect::<Vec<(i32, i32)>>();

    if ctx.debug {
        println!("The game is over.");
        for (i, &(points, turns)) in points.iter().enumerate() {
            let ref name = game.players[i].name;
            println!("{}: {} points in {} turns", name, points, turns);
        }
    }

    let high_score = points.iter().max_by_key(|pair| {
        (pair.0, pair.1 * -1)
    }).unwrap();

    let winners = points.iter().filter(|pair| *pair == high_score).collect::<Vec<_>>();
    return points.iter().map(|pair| {
        if pair == high_score {
            1.0 / (winners.len() as f32)
        } else{
            0.0
        }
    }).collect();
}

fn main() {
    let num_games = 1000;
    println!("Running {} games...", num_games);

    let players = vec![
        Box::new(RandomDecider) as Box<Decider>,
        Box::new(BigMoney) as Box<Decider>];

    let mut results = vec![0.0; 2];

    for _ in 0..num_games {
        let r = run_game(&players, false);
        for (i, score) in r.iter().enumerate() {
            results[i] += *score;
        }
    }
    println!("");
    for (i, score) in results.iter().enumerate() {
        println!("Player {} won {} game(s)", players[i].description(), score);
    }
}

#[test]
fn test_draw() {
    let mut ctx = EvalContext { debug: false, rng: thread_rng() };
    let mut p = fresh_player(&"Test Player".to_string());
    p.draw_cards(5, &mut ctx);
    assert!(p.hand.len() == 5);
    assert!(p.deck.len() == 5);
    assert!(p.all_cards().len() == 10);
    p.discard_hand(&mut ctx);
    assert!(p.hand.len() == 0);
    assert!(p.discard.len() == 5);
    assert!(p.all_cards().len() == 10);
    p.draw_cards(5, &mut ctx);
    assert!(p.deck.len() == 0);
    assert!(p.hand.len() == 5);
    assert!(p.discard.len() == 5);
    assert!(p.all_cards().len() == 10);
    p.discard_hand(&mut ctx);
    p.draw_cards(5, &mut ctx);
    assert!(p.discard.len() == 0);
    assert!(p.deck.len() == 5);
    assert!(p.hand.len() == 5);
    assert!(p.all_cards().len() == 10);
}

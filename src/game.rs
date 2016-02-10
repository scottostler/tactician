use rand::{Rng, XorShiftRng};
use std;
use std::collections::HashMap;

use cards;
use cards::{Card, CardIdentifier};
use util::{subtract_vector, randomly_seeded_weak_rng};

const EMPTY_PILES_FOR_GAME_END: i32 = 3;
const PLAYER_HAND_SIZE: usize = 5;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Phase {
    StartTurn,
    Action,
    BuyPlayTreasure,
    BuyPurchaseCard,
    Cleanup,
    EndTurn
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct PlayerIdentifier(pub i32);

#[derive(Clone)]
pub struct Player {
    identifier: PlayerIdentifier,
    name: String,
    hand: Vec<CardIdentifier>,
    discard: Vec<CardIdentifier>,
    deck: Vec<CardIdentifier>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DecisionType {
    PlayTreasures,
    BuyCard
}

#[derive(Clone)]
pub struct Decision {
    pub player: PlayerIdentifier,
    pub decision_type: DecisionType,
    pub choices: Vec<CardIdentifier>,
    pub range: (usize, usize),
}

pub trait Decider {
    fn description(&self) -> String;
    fn make_decision(&mut self, g: &Game) -> Vec<CardIdentifier>;
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
            println!("{} discarded {}", self.name, cards::card_names(&self.hand));
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
pub struct Game {
    pub turn: i32,
    pub active_player: PlayerIdentifier,
    pub phase: Phase,
    pub actions: i32,
    pub buys: i32,
    pub coins: i32,
    pub piles: HashMap<CardIdentifier, i32>,
    pub play_area: Vec<CardIdentifier>,
    pub players: Vec<Player>,
    pub pending_decision: Option<Decision>
}

pub struct EvalContext {
    pub rng: XorShiftRng,
    pub debug: bool
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

    pub fn is_game_over(&self) -> bool {
        if self.phase != Phase::EndTurn {
            return false;
        } else if self.piles[&cards::PROVINCE.identifier] == 0 {
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

    pub fn player_vp_and_turns(&self) -> Vec<(i32, i32)> {
        return self.players.iter().enumerate().map(|(i, p)| {
            let score = cards::score_cards(&p.all_cards());
            if i <= (self.active_player.0 as usize) {
                (score, self.turn)
            } else {
                (score, self.turn - 1)
            }
        }).collect::<Vec<(i32, i32)>>();
    }

    pub fn player_scores(&self) -> Vec<(PlayerIdentifier, f32)> {
        assert!(self.is_game_over());
        let points = self.player_vp_and_turns();
        let high_score = points.iter().max_by_key(|pair| {
            (pair.0, pair.1 * -1)
        }).unwrap();

        let winners = points.iter().filter(|pair| *pair == high_score).collect::<Vec<_>>();
        return self.players.iter().zip(points.iter()).map(|(player, pair)| {
            let score = if pair == high_score {
                1.0 / (winners.len() as f32)
            } else {
                0.0
            };
            (player.identifier, score)
        }).collect();
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

    pub fn advance_game(&mut self, ctx: &mut EvalContext) {
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
                    .iter().cloned().filter(|c| cards::lookup_card(c).is_treasure())
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
                        if num > 0 && cards::lookup_card(ci).cost <= available_coins {
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

    pub fn resolve_decision(&mut self, result: Vec<CardIdentifier>, ctx: &mut EvalContext) {
        let decision = self.pending_decision.take().expect("Game::resolve_decision called without pending decision");
        match decision.decision_type {
            DecisionType::BuyCard => {
                match result.first() {
                    Some(ci) => {
                        let c = cards::lookup_card(ci);
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
                    let cards = result.iter().map(|ci| cards::lookup_card(ci) ).collect::<Vec<&Card>>();

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

impl std::fmt::Debug for Game {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Turn {}, {}'s turn", self.turn, self.players[self.active_player.0 as usize].name)
    }
}

fn fresh_player(identifier: PlayerIdentifier, name: &String) -> Player {
    let mut discard = std::iter::repeat(cards::COPPER.identifier).take(7).collect::<Vec<CardIdentifier>>();
    discard.extend(std::iter::repeat(cards::ESTATE.identifier).take(3));
    return Player { identifier: identifier, name: name.clone(), hand: Vec::new(), deck: Vec::new(), discard: discard };
}

fn fresh_game(player_names: &Vec<String>) -> Game {
    let players = player_names.iter().enumerate().map(|(i, name)| {
            return fresh_player(PlayerIdentifier(i as i32), name);
        }).collect::<Vec<_>>();

    return Game {
        turn: 1,
        active_player: players.first().unwrap().identifier,
        phase: Phase::StartTurn,
        actions: 1,
        buys: 1,
        coins: 0,
        piles: cards::standard_piles(players.len() as i32),
        play_area: Vec::new(),
        players: players,
        pending_decision: None,
    };
}

pub fn run_game(players: &mut Vec<Box<Decider>>, debug: bool) -> Vec<f32> {
    let mut ctx = EvalContext { rng: randomly_seeded_weak_rng(), debug: debug };

    let player_names = players.iter().map(|d| d.description()).collect::<Vec<_>>();
    let mut game = fresh_game(&player_names);
    game.initialize_game(&mut ctx);

    while !game.is_game_over() {
        if game.pending_decision.is_some() {
            let player_idx = game.pending_decision.as_ref().unwrap().player.0 as usize;
            let choice = players[player_idx].make_decision(&game);
            game.resolve_decision(choice, &mut ctx);
        } else {
            game.advance_game(&mut ctx);
        }
    }

    if ctx.debug {
        let points = game.player_vp_and_turns();
        println!("The game is over.");
        for (i, &(points, turns)) in points.iter().enumerate() {
            let ref name = game.players[i].name;
            println!("{}: {} VP in {} turns", name, points, turns);
        }
    }

    return game.player_scores().iter().map(|&(_, score)| score).collect();
}

#[test]
fn test_draw() {
    let mut ctx = EvalContext { debug: false, rng: randomly_seeded_weak_rng() };
    let mut p = fresh_player(PlayerIdentifier(0), &"Test Player".to_string());
    p.draw_cards(5, &mut ctx);
    assert_eq!(p.hand.len(), 5);
    assert_eq!(p.deck.len(), 5);
    assert_eq!(p.all_cards().len(), 10);
    p.discard_hand(&mut ctx);
    assert_eq!(p.hand.len(), 0);
    assert_eq!(p.discard.len(), 5);
    assert_eq!(p.all_cards().len(), 10);
    p.draw_cards(5, &mut ctx);
    assert_eq!(p.deck.len(), 0);
    assert_eq!(p.hand.len(), 5);
    assert_eq!(p.discard.len(), 5);
    assert_eq!(p.all_cards().len(), 10);
    p.discard_hand(&mut ctx);
    p.draw_cards(5, &mut ctx);
    assert_eq!(p.discard.len(), 0);
    assert_eq!(p.deck.len(), 5);
    assert_eq!(p.hand.len(), 5);
    assert_eq!(p.all_cards().len(), 10);
}


use rand::{sample, Rng, XorShiftRng};

use cards;
use cards::{CardIdentifier};
use game::{Decider, DecisionType, Game};
use util;

pub struct BigMoney;

impl Decider for BigMoney {

    fn description(&self) -> String { return "Big Money".into(); }

    fn make_decision(&mut self, g: &Game) -> Vec<CardIdentifier> {
        let d = g.pending_decision.as_ref().expect("BigMoney::make_decision called without pending decision");
        match d.decision_type {
            DecisionType::PlayAction => panic!("BigMoney should not buy actions"),
            DecisionType::PlayTreasures => return d.choices.clone(),
            DecisionType::BuyCard => {
                let cs = g.coins;
                if cs >= cards::PROVINCE.cost {
                    vec![cards::PROVINCE.identifier]
                } else if cs >= cards::GOLD.cost {
                    vec![cards::GOLD.identifier]
                } else if cs >= cards::SILVER.cost {
                    vec![cards::SILVER.identifier]
                } else {
                    vec![]
                }
            },
            DecisionType::DiscardCards => {
                let mut cards = d.choices.clone();
                // Available in Rust 1.7
                // cards.sort_by_key(|c| cards::lookup_card(c).coin_value.unwrap_or(0));
                cards.sort_by(|a, b| {
                    let a_coins = cards::lookup_card(a).coin_value.unwrap_or(0);
                    let b_coins = cards::lookup_card(b).coin_value.unwrap_or(0);
                    a_coins.cmp(&b_coins)
                });
                cards.iter().take(d.range.0).cloned().collect()
            }
        }
    }
}

pub struct RandomDecider {
    rng: XorShiftRng
}

impl RandomDecider {
    #[allow(dead_code)]
    pub fn new() -> RandomDecider {
        RandomDecider { rng:util::randomly_seeded_weak_rng() }
    }
}

impl Decider for RandomDecider {

    fn description(&self) -> String { return "Random".into(); }

    fn make_decision(&mut self, g: &Game) -> Vec<CardIdentifier> {
        let d = g.pending_decision.as_ref().expect("BigMoney::make_decision called without pending decision");
        if d.decision_type == DecisionType::PlayTreasures {
            return d.choices.clone();
        }

        let n = match d.range.0 == d.range.1 {
            true => d.range.0,
            false => self.rng.gen_range(d.range.0, d.range.1 + 1) as usize,
        };
        return sample(&mut self.rng, d.choices.clone(), n);
    }
}

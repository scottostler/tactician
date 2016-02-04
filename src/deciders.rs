use rand::{thread_rng, Rng, sample};

use cards;
use cards::{CardIdentifier};
use game::{Decider, Decision, DecisionType, Game};

pub struct BigMoney;

impl Decider for BigMoney {

    fn description(&self) -> String { return "Big Money".into(); }

    fn make_decision(&self, d: &Decision, g: &Game) -> Vec<CardIdentifier> {
        match d.decision_type {
            DecisionType::PlayTreasures => return d.choices.clone(),
            DecisionType::BuyCard => {
                let cs = g.coins;
                if cs >= cards::PROVINCE.cost {
                    return vec![cards::PROVINCE.identifier];
                } else if cs >= cards::GOLD.cost {
                    return vec![cards::GOLD.identifier];
                } else if cs >= cards::SILVER.cost {
                    return vec![cards::SILVER.identifier];
                } else {
                    return vec![];
                }
            }
        }
    }
}

pub struct RandomDecider;

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
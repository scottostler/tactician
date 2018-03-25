use itertools::Itertools;

use cards::CardIdentifier;

use game::{Decider, Decision, DecisionType, EvalContext, Game, PlayerIdentifier};
use tree_search;

fn hard_coded_decision(d: &Decision) -> Option<Vec<CardIdentifier>> {
    match d.decision_type {
        DecisionType::PlayTreasures => Some(d.choices.clone()),
        _ => None,
    }
}

impl tree_search::SearchableState for Game {
    type P = PlayerIdentifier;
    type M = Vec<CardIdentifier>;
    type C = EvalContext;

    fn game_result(&self) -> Option<tree_search::Winners<Self::P>> {
        if self.is_game_over() {
            let scores = self.player_scores();
            let winners = scores
                .iter()
                .filter_map(|&(pid, score)| if score > 0.0 { Some(pid) } else { None })
                .collect::<Vec<PlayerIdentifier>>();
            Some(tree_search::Winners(winners))
        } else {
            None
        }
    }

    fn all_players(&self) -> Vec<Self::P> {
        vec![PlayerIdentifier(0), PlayerIdentifier(1)]
    }

    fn active_player(&self) -> Option<Self::P> {
        self.pending_decision.as_ref().map(|d| d.player.clone())
    }

    fn all_moves(&self) -> Vec<Self::M> {
        if self.is_game_over() {
            return vec![];
        }

        let d = self.pending_decision
            .as_ref()
            .expect("Game::all_moves called without pending decision");

        if let Some(choice) = hard_coded_decision(&d) {
            return vec![choice];
        }

        let mut ret: Vec<Self::M> = vec![];
        for i in d.range.0..d.range.1 + 1 {
            if i == 0 {
                ret.push(vec![]);
                continue;
            } else if i == 1 {
                for c in &d.choices {
                    ret.push(vec![c.clone()]);
                }
                continue;
            }

            let combinations = d.choices.iter().combinations(i);
            for c in combinations {
                let mut v = Vec::with_capacity(c.len());
                for x in c {
                    v.push(*x);
                }
                ret.push(v);
            }
        }
        ret
    }

    fn make_move(&self, choice: Self::M, ctx: &mut Self::C) -> Self {
        let mut game_copy = self.clone();
        game_copy.resolve_decision(choice, ctx);

        while !game_copy.is_game_over() && game_copy.pending_decision.is_none() {
            game_copy.advance_game(ctx);
        }

        game_copy
    }

    fn make_move_mut(&mut self, choice: Self::M, ctx: &mut Self::C) {
        self.resolve_decision(choice, ctx);
        while !self.is_game_over() && self.pending_decision.is_none() {
            self.advance_game(ctx);
        }
    }

    fn printable_player_identifier(&self, p: &Self::P) -> String {
        self.players[p.0 as usize].name.clone()
    }
}

pub struct SearchDecider {
    pub ctx: EvalContext,
    pub debug: bool,
    pub iterations: i32,
}

impl Decider for SearchDecider {
    fn description(&self) -> String {
        return "Tactician".into();
    }

    fn make_decision(&mut self, g: &Game) -> Vec<CardIdentifier> {
        {
            let d = g.pending_decision
                .as_ref()
                .expect("SearchDecider::make_decision called without pending decision");
            if let Some(choice) = hard_coded_decision(&d) {
                return choice;
            }
        }

        let best_move =
            tree_search::find_best_move(g.clone(), self.iterations, &mut self.ctx, self.debug);
        best_move
    }
}

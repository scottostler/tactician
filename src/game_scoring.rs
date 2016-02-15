use cards;
use game::{EMPTY_PILES_FOR_GAME_END, Game, Phase, PlayerIdentifier};

impl Game {

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

}
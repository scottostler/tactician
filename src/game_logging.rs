use itertools::Itertools;
use cards;
use game::{EvalContext, Game, EMPTY_PILES_FOR_GAME_END};

impl Game {
    pub fn print_turn_start_summary(&self, ctx: &mut EvalContext) {
        if !ctx.debug {
            return;
        }

        let ref player = self.players[self.active_player.0 as usize];
        println!("\n----- Turn {}, {} -----", self.turn, player.name);

        let vp_and_turns = self.player_vp_and_turns();
        let player_vp_pairs = self.players.iter().zip(vp_and_turns);

        for (player, (vp, _)) in player_vp_pairs {
            println!("- {}: {} VP", player.name, vp)
        }

        let provinces_left = self.piles[&cards::PROVINCE.identifier];
        if provinces_left == 1 {
            println!("- 1 Province left");
        } else {
            println!("- {} Provinces left", provinces_left);
        }

        let non_province_pile_counts = self.piles
            .iter()
            .filter(|&(card, _)| *card != cards::PROVINCE.identifier)
            .sorted_by_key(|&(_, count)| count);

        let cards_to_empty_string = non_province_pile_counts
            .iter()
            .take(EMPTY_PILES_FOR_GAME_END as usize)
            .map(|&(card, count)| {
                let card = cards::lookup_card(card);
                if *count == 0 {
                    format!("**{}**", card.name)
                } else {
                    format!("{} ({})", card.name, count)
                }
            })
            .join(", ");

        let count_to_empty: i32 = non_province_pile_counts
            .iter()
            .take(EMPTY_PILES_FOR_GAME_END as usize)
            .map(|&(_, count)| count)
            .sum();

        println!("- {} other cards to empty piles", count_to_empty);
        println!("  {}", cards_to_empty_string);
        println!();
    }
}

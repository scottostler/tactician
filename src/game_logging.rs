use itertools::Itertools;
use cards;
use game::{EvalContext, Game, EMPTY_PILES_FOR_GAME_END};

impl Game {

    pub fn print_turn_start_summary(&self, ctx: &mut EvalContext) {
        if !ctx.debug {
            return
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

        // TODO: Needs sorted_by_key from itertools update
        // let piles_to_empty = self.piles
        //     .iter()
        //     .filter(|&(card, count)| *card != cards::PROVINCE.identifier)
        //     .sorted_by_key(|(card, count)| -count)  
        //     .take(EMPTY_PILES_FOR_GAME_END);

        // println!()
    }

}
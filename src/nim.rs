use tree_search::*;

#[derive(Clone, Debug)]
pub struct NimState {
    total: i32,
    player_turn: i32
}

impl SearchableState for NimState {
    type P = i32;
    type M = i32;
    type C = ();
    
    fn game_result(&self) -> Option<Winners<Self::P>> {
        if self.total == 0 {
            let last_player = (self.player_turn + 1) % 2;
            return Some(Winners(vec![last_player]));
        } else {
            return None
        }
    }
    
    fn all_players(&self) -> Vec<Self::P> {
        vec![0, 1]
    }
    
    fn active_player(&self) -> Option<Self::P> {
        Some(self.player_turn)
    }
    
    fn all_moves(&self) -> Vec<Self::M> {
        return (1..4).into_iter().filter(|&i| i <= self.total).collect::<Vec<_>>();
    }
    
    fn make_move(&self, choice:Self::M, _: &mut Self::C) -> Self {
        return NimState {
            total: self.total - choice,
            player_turn: (self.player_turn + 1) % 2
        };
    }
    
    fn make_move_mut(&mut self, choice:Self::M, _: &mut Self::C) {
        self.total -= choice;
        self.player_turn = (self.player_turn + 1) % 2;
    }
}

#[cfg(test)]
mod tests {
    
    use tree_search;
    use nim::*;
    
    #[test]
    fn test_nim_search() {
        let start_state = NimState { total: 15, player_turn: 0 };
        let best_move = tree_search::find_best_move(start_state, 10000, &mut (), false);
        assert_eq!(best_move, 3);
    }
}

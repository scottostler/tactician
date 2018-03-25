use tree_search::{NodeStats, SearchNode, SearchableState};

impl<T: SearchableState> SearchNode<T> {
    pub fn print_debug_move_tree(&self) {
        println!("  {:?} --", self.state);
        if let Some(p) = self.state.active_player() {
            println!(
                "    Moves for {}: ",
                self.state.printable_player_identifier(&p)
            );

            self.print_child_move_stats();

            if !self.children.is_empty() {
                let child = self.most_visited_child();
                child.borrow().print_debug_move_tree();
            } else {
                println!("    ...tree is exhausted");
            }
        } else {
            println!("    ...game is over");
        }
    }

    pub fn print_child_move_stats(&self) {
        let mut child_stats: Vec<NodeStats<T>> =
            self.children.iter().map(|c| c.borrow().stats()).collect();

        // Reverse so in descending order
        child_stats.sort_by(|a, b| (b.percent_won).partial_cmp(&a.percent_won).unwrap());

        for stat in child_stats.iter() {
            println!(
                "    {:?}: won {} / {} ({:.2}%) visits",
                stat.last_move
                    .as_ref()
                    .expect("children should have last move"),
                stat.wins,
                stat.visits,
                100.0 * stat.percent_won as f32
            );
        }
    }
}

use rand::{Rng, XorShiftRng};
use std::fmt::Debug;
use std::rc::{Rc, Weak};
use std::cell::RefCell;

use util;

#[derive(Debug, Eq, PartialEq)]
pub struct Winners<P>(pub Vec<P>);

pub type WeakNodeRef<T> = Weak<RefCell<SearchNode<T>>>;
pub type NodeRef<T> = Rc<RefCell<SearchNode<T>>>;

pub trait SearchableState: Clone + Debug {
    type P: Clone + PartialEq + Debug;
    type M: Clone + Debug;
    type C;

    fn game_result(&self) -> Option<Winners<Self::P>>;
    fn all_players(&self) -> Vec<Self::P>;
    fn active_player(&self) -> Option<Self::P>;
    fn all_moves(&self) -> Vec<Self::M>;
    fn make_move(&self, Self::M, &mut Self::C) -> Self;
    fn make_move_mut(&mut self, Self::M, &mut Self::C);

    fn printable_player_identifier(&self, p: &Self::P) -> String;
}

#[derive(Debug)]
pub struct SearchNode<T: SearchableState> {
    state: T,
    wins: f32,
    visits: i32,
    last_move: Option<T::M>,
    untried_moves: Vec<T::M>,
    player_just_moved: T::P,
    parent: Option<WeakNodeRef<T>>,
    children: Vec<NodeRef<T>>,
}

impl<T: SearchableState> SearchNode<T> {
    fn print_debug_move_tree(&self) {
        println!("  {:?} --", self.state);
        if let Some(p) = self.state.active_player() {
            println!(
                "    Moves for {}: ",
                self.state.printable_player_identifier(&p)
            );
            for c in &self.children {
                let c = c.borrow();
                println!(
                    "    {:?}: won {} / {} ({:.2}%) visits",
                    c.last_move
                        .as_ref()
                        .expect("children should have last move"),
                    c.wins,
                    c.visits,
                    100.0 * c.wins / c.visits as f32
                );
            }

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

    fn expectation(&self, parent_visits: f32) -> f32 {
        let f_visits = self.visits as f32;
        let payout = self.wins / f_visits;
        let confidence = (2.0 * parent_visits.ln() / f_visits).sqrt();
        payout + confidence
    }

    pub fn most_visited_child(&self) -> NodeRef<T> {
        self.children
            .iter()
            .max_by_key(|&c| c.borrow().visits)
            .expect("most_visited_child() called on terminal node")
            .clone()
    }

    pub fn select_most_promising_child(&mut self) -> NodeRef<T> {
        let parent_visits = self.visits as f32;
        self.children.sort_by(|a, b| {
            let a_exp = a.borrow().expectation(parent_visits);
            let b_exp = b.borrow().expectation(parent_visits);
            match a_exp.partial_cmp(&b_exp) {
                Some(o) => o.reverse(), // Sort most promising first
                None => panic!("SearchNode::select_most_promising_child failed with non-total comparison of {} vs {}", a_exp, b_exp)
            }
        });
        self.children
            .first()
            .expect("SearchNode::select_most_promising_child failed: no children")
            .clone()
    }

    fn update_with_result(&mut self, result: &Winners<T::P>) {
        self.visits += 1;
        if result.0.contains(&self.player_just_moved) {
            self.wins += 1.0 / result.0.len() as f32;
        }
    }

    fn ancestors(&self) -> Vec<NodeRef<T>> {
        let mut vector = vec![];
        fn walk<T: SearchableState>(parent_ref: &Option<WeakNodeRef<T>>, v: &mut Vec<NodeRef<T>>) {
            match parent_ref {
                &Some(ref p) => match p.upgrade() {
                    Some(n) => {
                        v.push(n.clone());
                        walk(&n.borrow().parent, v);
                    }
                    None => {}
                },
                &None => {}
            };
        }

        walk(&self.parent, &mut vector);
        return vector;
    }
}

fn expand_node_by_move<T: SearchableState>(
    node_ref: NodeRef<T>,
    move_idx: usize,
    ctx: &mut T::C,
) -> NodeRef<T> {
    let mut node = node_ref.borrow_mut();
    let picked_move = node.untried_moves[move_idx].clone();
    let new_state = node.state.make_move(picked_move.clone(), ctx);
    let all_moves = new_state.all_moves();

    let new_node = SearchNode {
        state: new_state,
        wins: 0.0,
        visits: 0,
        last_move: Some(picked_move),
        untried_moves: all_moves,
        player_just_moved: node.state
            .active_player()
            .expect("State with move must have active player"),
        parent: Some(Rc::downgrade(&node_ref)),
        children: vec![],
    };

    let new_node_cell = Rc::new(RefCell::new(new_node));
    node.untried_moves.remove(move_idx);
    node.children.push(new_node_cell.clone());
    new_node_cell
}

fn best_unexplored_node<T: SearchableState>(node_ref: &NodeRef<T>) -> NodeRef<T> {
    let mut node = node_ref.borrow_mut();
    if node.untried_moves.is_empty() && !node.children.is_empty() {
        let child_ref = node.select_most_promising_child();
        best_unexplored_node(&child_ref)
    } else {
        node_ref.clone()
    }
}

fn choose_random_move<T: SearchableState>(state: &T, rng: &mut XorShiftRng) -> Option<T::M> {
    let possible_moves = state.all_moves();
    if possible_moves.is_empty() {
        None
    } else {
        rng.choose(&possible_moves).cloned()
    }
}

fn simulate_until_terminal<T: SearchableState>(
    state: T,
    rng: &mut XorShiftRng,
    ctx: &mut T::C,
) -> T {
    let mut mut_state = state;
    while let Some(m) = choose_random_move(&mut_state, rng) {
        mut_state.make_move_mut(m, ctx);
    }
    mut_state
}

pub fn find_best_move<T: SearchableState>(
    root_state: T,
    max_iters: i32,
    ctx: &mut T::C,
    debug: bool,
) -> T::M {
    let mut rng = util::randomly_seeded_weak_rng();
    let untried = root_state.all_moves();

    // Start with last player as having moved. Not meaningful for >2P games.
    let just_moved: T::P = root_state
        .all_players()
        .last()
        .cloned()
        .expect("Players must not be empty");
    let root_node = Rc::new(RefCell::new(SearchNode {
        state: root_state,
        wins: 0.0,
        visits: 0,
        last_move: None,
        untried_moves: untried,
        player_just_moved: just_moved,
        parent: None,
        children: vec![],
    }));

    for _ in 0..max_iters {
        // Select
        let mut node_ref = best_unexplored_node(&root_node);

        // Expand
        if !node_ref.borrow().untried_moves.is_empty() {
            let move_idx = rng.gen_range(0, node_ref.borrow().untried_moves.len());
            let child_ref = expand_node_by_move(node_ref, move_idx, ctx);
            node_ref = child_ref;
        }

        // Rollout
        let start_state = node_ref.borrow().state.clone();
        let end_state = simulate_until_terminal(start_state, &mut rng, ctx);
        let result = end_state
            .game_result()
            .expect("Terminal game state is missing a result");

        // Backpropagate
        node_ref.borrow_mut().update_with_result(&result);
        for n_ref in node_ref.borrow().ancestors() {
            n_ref.borrow_mut().update_with_result(&result);
        }
    }

    let borrowed_root = root_node.borrow();
    if debug {
        borrowed_root.print_debug_move_tree();
    }

    let best_child = borrowed_root.most_visited_child();
    let best_move = best_child.borrow().last_move.as_ref().unwrap().clone();
    best_move
}

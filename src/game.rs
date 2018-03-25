use rand::{Rng, XorShiftRng};
use std;
use std::collections::HashMap;

use cards;
use cards::{Card, CardAction, CardIdentifier, CardReaction, CardType, DiscardEffect, EffectTarget,
            GainDestination, TrashFollowup};
use util::{randomly_seeded_weak_rng, subtract_vector};

pub const EMPTY_PILES_FOR_GAME_END: i32 = 3;
pub const PLAYER_HAND_SIZE: usize = 5;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Phase {
    StartTurn,
    Action,
    BuyPlayTreasure,
    BuyPurchaseCard,
    Cleanup,
    EndTurn,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct PlayerIdentifier(pub u8);

// ActionIdentifiers are used to track an instance of a played action,
// such as to record when a player has revealed a Moat to a specific attack.
// If an action is played multiple times by a card like Throne Room, each play
// has its own ActionIdentifier.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ActionIdentifier(pub u32);

impl ActionIdentifier {
    fn new() -> ActionIdentifier {
        ActionIdentifier(0)
    }

    fn increment(&self) -> ActionIdentifier {
        ActionIdentifier(self.0 + 1)
    }
}

#[derive(Clone)]
pub struct Player {
    pub identifier: PlayerIdentifier,
    pub name: String,
    pub hand: Vec<CardIdentifier>,
    pub discard: Vec<CardIdentifier>,
    pub deck: Vec<CardIdentifier>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DecisionType {
    PlayAction,
    PlayTreasures,
    BuyCard,
    GainCard(GainDestination),
    DiscardCards(Option<DiscardEffect>),
    TrashCards(Option<TrashFollowup>),
    RevealReaction(ActionIdentifier),
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
    fn draw_cards(&mut self, n: usize, ctx: &mut EvalContext) {
        assert!(n > 0, "Drawing 0 cards does nothing");
        let mut drawn = if self.deck.len() >= n {
            let pivot = self.deck.len() - n;
            self.deck.split_off(pivot)
        } else {
            let mut first_draw: Vec<CardIdentifier> = self.deck.clone();

            ctx.rng.shuffle(&mut self.discard);
            self.deck = self.discard.clone();
            self.discard.clear();

            if ctx.debug {
                println!("{} shuffles", self.name);
            }

            let second_n = std::cmp::min(self.deck.len(), n - first_draw.len());
            let pivot = self.deck.len() - second_n;
            let mut second_draw = self.deck.split_off(pivot);
            first_draw.append(&mut second_draw);
            first_draw
        };

        if ctx.debug {
            println!("{} draws {} cards", self.name, drawn.len());
        }

        self.hand.append(&mut drawn);
    }

    fn discard_hand(&mut self, ctx: &mut EvalContext) {
        if ctx.debug {
            println!("{} discards {}", self.name, cards::card_names(&self.hand));
        }

        self.discard.extend(&self.hand);
        self.hand.clear();
    }

    pub fn all_cards(&self) -> Vec<CardIdentifier> {
        let mut ret = Vec::new();
        ret.extend(&self.hand);
        ret.extend(&self.deck);
        ret.extend(&self.discard);
        return ret;
    }
}

#[derive(Clone)]
pub enum QueuedEffect {
    ActionEffect(PlayerIdentifier, ActionIdentifier, CardAction),
    ReactOption(PlayerIdentifier, ActionIdentifier),
}

#[derive(Clone)]
pub struct Game {
    pub turn: i32,
    pub active_player: PlayerIdentifier,
    pub phase: Phase,
    pub actions: i32,
    pub buys: i32,
    pub coins: i32,
    pub current_action_identifier: ActionIdentifier,
    pub piles: HashMap<CardIdentifier, i32>,
    pub play_area: Vec<CardIdentifier>,
    pub trash_pile: Vec<CardIdentifier>,
    pub players: Vec<Player>,
    pub pending_decision: Option<Decision>,
    pub pending_effects: Vec<QueuedEffect>,
}

pub struct EvalContext {
    pub rng: XorShiftRng,
    pub debug: bool,
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

    fn gainable_cards_costing(&self, cost_range: (i32, i32)) -> Vec<CardIdentifier> {
        let mut gainable = vec![];
        for (ci, &num) in self.piles.iter() {
            if num > 0 {
                let cost = cards::lookup_card(ci).cost;
                if cost >= cost_range.0 && cost <= cost_range.1 {
                    gainable.push(*ci);
                }
            }
        }
        gainable
    }

    fn player_draws_cards(&mut self, pid: PlayerIdentifier, n: i32, ctx: &mut EvalContext) {
        let ref mut player = self.players[pid.0 as usize];
        player.draw_cards(n as usize, ctx);
    }

    fn player_discards_to(&mut self, pid: PlayerIdentifier, n: i32, _: &mut EvalContext) {
        let ref mut player = self.players[pid.0 as usize];
        if player.hand.len() > n as usize {
            let discard_count = (player.hand.len() as i32 - n) as usize;
            self.pending_decision = Some(Decision {
                player: pid,
                decision_type: DecisionType::DiscardCards(None),
                choices: player.hand.clone(),
                range: (discard_count, discard_count),
            })
        }
    }

    fn player_discards(
        &mut self,
        pid: PlayerIdentifier,
        cards: Vec<CardIdentifier>,
        maybe_effect: Option<DiscardEffect>,
        ctx: &mut EvalContext,
    ) {
        {
            let ref mut player = self.players[pid.0 as usize];
            player.discard.extend(&cards);
            subtract_vector::<CardIdentifier>(&mut player.hand, &cards);
            if ctx.debug {
                println!("{} discards {}", player.name, cards::card_names(&cards));
            }
        }

        if let Some(maybe_effect) = maybe_effect {
            match maybe_effect {
                DiscardEffect::DrawPerDiscard => {
                    self.player_draws_cards(pid, cards.len() as i32, ctx)
                }
            }
        }
    }

    fn player_picks_gain(
        &mut self,
        pid: PlayerIdentifier,
        cost_range: (i32, i32),
        _: &mut EvalContext,
    ) {
        let cards = self.gainable_cards_costing(cost_range);
        if !cards.is_empty() {
            self.pending_decision = Some(Decision {
                player: pid,
                decision_type: DecisionType::GainCard(GainDestination::GainToDiscard),
                choices: cards,
                range: (1, 1),
            });
        }
    }

    fn offer_player_trash(
        &mut self,
        pid: PlayerIdentifier,
        maybe_card_type: Option<CardType>,
        followup: Option<TrashFollowup>,
        _: &mut EvalContext,
    ) {
        let ref player = self.players[pid.0 as usize];

        let trashable = if let Some(card_type) = maybe_card_type {
            cards::filter_by_type(&player.hand, &card_type)
        } else {
            player.hand.clone()
        };

        if !trashable.is_empty() {
            self.pending_decision = Some(Decision {
                player: pid,
                decision_type: DecisionType::TrashCards(followup),
                choices: trashable,
                range: (1, 1),
            });
        }
    }

    fn offer_player_discard(
        &mut self,
        pid: PlayerIdentifier,
        discard_effect: DiscardEffect,
        _: &mut EvalContext,
    ) {
        let ref player = self.players[pid.0 as usize];
        if player.hand.is_empty() {
            return;
        }

        self.pending_decision = Some(Decision {
            player: pid,
            decision_type: DecisionType::DiscardCards(Some(discard_effect)),
            choices: player.hand.clone(),
            range: (0, player.hand.len()),
        })
    }

    fn next_turn(&mut self) {
        if self.active_player.0 + 1 == self.players.len() as u8 {
            self.turn += 1;
            self.active_player = PlayerIdentifier(0);
        } else {
            self.active_player = PlayerIdentifier(self.active_player.0 + 1);
        }

        self.phase = Phase::StartTurn;
        self.actions = 1;
        self.buys = 1;
        self.coins = 0;
        self.current_action_identifier = ActionIdentifier::new();
    }

    fn process_effect(&mut self, e: QueuedEffect, ctx: &mut EvalContext) {
        match e {
            QueuedEffect::ActionEffect(pid, _, ca) => match ca {
                CardAction::DrawCards(n) => self.player_draws_cards(pid, n, ctx),
                CardAction::PlusActions(n) => self.actions += n,
                CardAction::PlusBuys(n) => self.buys += n,
                CardAction::PlusCoins(n) => self.coins += n,
                CardAction::OpponentsDiscardTo(n) => self.player_discards_to(pid, n, ctx),
                CardAction::GainCardCostingUpto(n) => self.player_picks_gain(pid, (0, n), ctx),
                CardAction::TrashCards(card_type, followup) => {
                    self.offer_player_trash(pid, card_type, followup, ctx)
                }
                CardAction::DiscardForEffect(discard_effect) => {
                    self.offer_player_discard(pid, discard_effect, ctx)
                }
            },
            QueuedEffect::ReactOption(pid, aid) => {
                let reactions =
                    cards::filter_by_type(&self.players[pid.0 as usize].hand, &CardType::Reaction);
                if !reactions.is_empty() {
                    self.pending_decision = Some(Decision {
                        player: pid,
                        decision_type: DecisionType::RevealReaction(aid),
                        choices: reactions.clone(),
                        range: (0, 1),
                    });
                }
            }
        }
    }

    pub fn advance_game(&mut self, ctx: &mut EvalContext) {
        assert!(
            self.pending_decision.is_none(),
            "Can't advance game with pending decision"
        );

        if !self.pending_effects.is_empty() {
            let e = self.pending_effects.remove(0);
            self.process_effect(e, ctx);
            return;
        }

        match self.phase {
            Phase::StartTurn => {
                self.print_turn_start_summary(ctx);
                self.phase = Phase::Action;
            }
            Phase::Action => {
                if self.actions == 0 {
                    self.phase = Phase::BuyPlayTreasure;
                    return;
                }

                let actions = self.players[self.active_player.0 as usize]
                    .hand
                    .iter()
                    .filter(|c| cards::lookup_card(c).is_action())
                    .cloned()
                    .collect::<Vec<CardIdentifier>>();

                if actions.is_empty() {
                    self.phase = Phase::BuyPlayTreasure;
                    return;
                }

                self.pending_decision = Some(Decision {
                    player: self.active_player,
                    decision_type: DecisionType::PlayAction,
                    choices: actions,
                    range: (0, 1),
                });
            }
            Phase::BuyPlayTreasure => {
                let treasures = self.players[self.active_player.0 as usize]
                    .hand
                    .iter()
                    .filter(|c| cards::lookup_card(c).is_treasure())
                    .cloned()
                    .collect::<Vec<CardIdentifier>>();

                if treasures.is_empty() {
                    self.phase = Phase::BuyPurchaseCard;
                } else {
                    let treasure_len = treasures.len();
                    self.pending_decision = Some(Decision {
                        player: self.active_player,
                        decision_type: DecisionType::PlayTreasures,
                        choices: treasures,
                        range: (0, treasure_len),
                    });
                }
            }
            Phase::BuyPurchaseCard => {
                if self.buys == 0 {
                    self.phase = Phase::Cleanup;
                } else {
                    let buyable = self.gainable_cards_costing((0, self.coins));
                    self.pending_decision = Some(Decision {
                        player: self.active_player,
                        decision_type: DecisionType::BuyCard,
                        choices: buyable,
                        range: (0, 1),
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

    fn gain_card(
        &mut self,
        player: PlayerIdentifier,
        ci: &CardIdentifier,
        dest: GainDestination,
        ctx: &mut EvalContext,
    ) {
        assert!(self.piles[ci] > 0, "Pile must not be empty");
        match self.piles.get_mut(ci) {
            Some(l) => *l -= 1,
            None => panic!("Cannot find pile for {}", cards::lookup_card(ci).name),
        }

        {
            let ref mut player = self.players[player.0 as usize];
            match dest {
                GainDestination::GainToDiscard => player.discard.push(*ci),
                GainDestination::GainToHand => player.hand.push(*ci),
            }
        }

        if ctx.debug {
            let c = cards::lookup_card(ci);
            println!("{} gains {}", self.players[player.0 as usize].name, c.name);
        }
    }

    fn buy_card(&mut self, player: PlayerIdentifier, ci: &CardIdentifier, ctx: &mut EvalContext) {
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
        self.players[player.0 as usize].discard.push(*ci);

        if ctx.debug {
            println!("{} buys {}", self.players[player.0 as usize].name, c.name);
        }
    }

    fn replace_card_by_cost(
        &mut self,
        pid: PlayerIdentifier,
        trashed: &Card,
        plus_cost: i32,
        maybe_card_type: Option<CardType>,
        dest: GainDestination,
        _: &mut EvalContext,
    ) {
        let mut gainable = self.gainable_cards_costing((0, trashed.cost + plus_cost));
        if let Some(card_type) = maybe_card_type {
            gainable = cards::filter_by_type(&gainable, &card_type);
        }

        if !gainable.is_empty() {
            self.pending_decision = Some(Decision {
                player: pid,
                decision_type: DecisionType::GainCard(dest),
                choices: gainable,
                range: (1, 1),
            });
        }
    }

    fn trash_cards(
        &mut self,
        pid: PlayerIdentifier,
        cards: Vec<CardIdentifier>,
        maybe_followup: Option<TrashFollowup>,
        ctx: &mut EvalContext,
    ) {
        assert!(!cards.is_empty(), "Game::trash_cards called with no cards");
        {
            let ref mut player = self.players[pid.0 as usize];
            subtract_vector(&mut player.hand, &cards);
        }

        self.trash_pile.extend(&cards);

        if ctx.debug {
            println!(
                "{} trashes {}",
                self.players[pid.0 as usize].name,
                cards::card_names(&cards)
            );
        }

        if let Some(followup) = maybe_followup {
            match followup {
                TrashFollowup::ReplaceByCost(maybe_card_type, plus_cost, dest) => {
                    assert_eq!(cards.len(), 1);
                    let trashed = cards::lookup_card(cards.first().unwrap());
                    self.replace_card_by_cost(pid, trashed, plus_cost, maybe_card_type, dest, ctx);
                }
            }
        }
    }

    fn player_reveals_reaction(
        &mut self,
        pid: PlayerIdentifier,
        c: &CardIdentifier,
        aid: ActionIdentifier,
        ctx: &mut EvalContext,
    ) {
        let reaction = cards::lookup_card(c);
        if ctx.debug {
            println!(
                "{} reveals {}",
                self.players[pid.0 as usize].name, reaction.name
            );
        }

        if let Some(ref rx_effect) = reaction.reaction_effect {
            match rx_effect {
                &CardReaction::AttackImmunity => {
                    self.pending_effects.retain(|queued_effect| {
                        if let &QueuedEffect::ActionEffect(ref e_pid, ref e_aid, _) = queued_effect
                        {
                            !(*e_pid == pid && *e_aid == aid)
                        } else {
                            true
                        }
                    });
                }
            }
        } else {
            panic!(
                "Card revealed to reaction is not a reaction: {}",
                reaction.name
            );
        }
    }

    fn players_for_target(
        &self,
        target: EffectTarget,
        active_player: PlayerIdentifier,
    ) -> Vec<PlayerIdentifier> {
        match target {
            EffectTarget::ActivePlayer => vec![active_player],
            EffectTarget::Opponents => {
                let num_players = self.players.len();
                (1..num_players)
                    .map(|i| PlayerIdentifier(((i + active_player.0 as usize) % num_players) as u8))
                    .collect()
            }
            EffectTarget::AllPlayers => {
                let num_players = self.players.len();
                (0..num_players)
                    .map(|i| PlayerIdentifier(((i + active_player.0 as usize) % num_players) as u8))
                    .collect()
            }
        }
    }

    fn queue_card_effects(
        &mut self,
        pid: PlayerIdentifier,
        aid: ActionIdentifier,
        action: &CardAction,
    ) {
        let target = cards::target_for_action(&action);
        for target_pid in self.players_for_target(target, pid) {
            self.pending_effects.push(QueuedEffect::ActionEffect(
                target_pid,
                aid.clone(),
                action.clone(),
            ));
        }
    }

    fn play_action(
        &mut self,
        pid: PlayerIdentifier,
        action: &CardIdentifier,
        ctx: &mut EvalContext,
    ) {
        assert!(self.actions > 0, "Must have an action");
        assert_eq!(self.phase, Phase::Action);
        assert_eq!(pid, self.active_player);

        {
            let ref mut player = self.players[pid.0 as usize];
            if ctx.debug {
                println!("{} plays {}", player.name, action);
            }

            let hand_idx = player
                .hand
                .iter()
                .position(|v| *v == *action)
                .expect("Player doesn't have card in hand");
            player.hand.remove(hand_idx);
        }

        self.play_area.push(*action);
        self.current_action_identifier = self.current_action_identifier.increment();
        let aid = self.current_action_identifier.clone();

        let card = cards::lookup_card(action);

        if card.is_attack {
            for target_pid in self.players_for_target(EffectTarget::Opponents, pid) {
                self.pending_effects
                    .push(QueuedEffect::ReactOption(target_pid, aid));
            }
        }

        for e in &card.action_effects {
            self.queue_card_effects(pid, aid, e);
        }
    }

    fn play_treasures(
        &mut self,
        pid: PlayerIdentifier,
        result: &Vec<CardIdentifier>,
        ctx: &mut EvalContext,
    ) {
        for c in result.iter().map(|ci| cards::lookup_card(ci)) {
            assert!(c.is_treasure(), "Can only play treasures");
            self.coins += c.coin_value.unwrap();
        }

        let ref mut player = self.players[pid.0 as usize];

        if ctx.debug {
            println!("{} plays {}", player.name, cards::card_names(result));
        }

        self.play_area.extend(result);
        subtract_vector::<CardIdentifier>(&mut player.hand, &result);
    }

    pub fn resolve_decision(&mut self, result: Vec<CardIdentifier>, ctx: &mut EvalContext) {
        let decision = self.pending_decision
            .take()
            .expect("Game::resolve_decision called without pending decision");
        match decision.decision_type {
            DecisionType::PlayAction => {
                assert!(result.len() <= 1, "Can only play at most one action");
                match result.first() {
                    Some(ci) => self.play_action(decision.player, ci, ctx),
                    None => self.phase = Phase::BuyPlayTreasure,
                }
            }
            DecisionType::PlayTreasures => {
                if !result.is_empty() {
                    self.play_treasures(decision.player, &result, ctx);
                }
                self.phase = Phase::BuyPurchaseCard;
            }

            DecisionType::BuyCard => {
                assert!(result.len() <= 1, "Can only buy at most one card");
                if let Some(c) = result.first() {
                    self.buy_card(decision.player, c, ctx);
                } else {
                    self.phase = Phase::Cleanup;
                }
            }
            DecisionType::DiscardCards(maybe_followup) => {
                if !result.is_empty() {
                    self.player_discards(decision.player, result, maybe_followup, ctx);
                }
            }
            DecisionType::GainCard(dest) => {
                assert!(result.len() <= 1, "Can only gain at most one card");
                if let Some(c) = result.first() {
                    self.gain_card(decision.player, c, dest, ctx);
                }
            }
            DecisionType::RevealReaction(aid) => {
                if let Some(c) = result.first() {
                    self.player_reveals_reaction(decision.player, c, aid, ctx);
                }
            }
            DecisionType::TrashCards(maybe_followup) => {
                if !result.is_empty() {
                    self.trash_cards(decision.player, result, maybe_followup, ctx);
                }
            }
        }
    }
}

impl std::fmt::Debug for Game {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "Turn {}, {}'s turn",
            self.turn, self.players[self.active_player.0 as usize].name
        )
    }
}

pub fn fresh_player(identifier: PlayerIdentifier, name: &String) -> Player {
    let mut discard = std::iter::repeat(cards::COPPER.identifier)
        .take(7)
        .collect::<Vec<CardIdentifier>>();
    discard.extend(std::iter::repeat(cards::ESTATE.identifier).take(3));
    return Player {
        identifier: identifier,
        name: name.clone(),
        hand: Vec::new(),
        deck: Vec::new(),
        discard: discard,
    };
}

pub fn fresh_game(player_names: &Vec<String>) -> Game {
    let players = player_names
        .iter()
        .enumerate()
        .map(|(i, name)| {
            return fresh_player(PlayerIdentifier(i as u8), name);
        })
        .collect::<Vec<_>>();

    return Game {
        turn: 1,
        active_player: players.first().unwrap().identifier,
        phase: Phase::StartTurn,
        actions: 1,
        buys: 1,
        coins: 0,
        current_action_identifier: ActionIdentifier::new(),
        piles: cards::standard_piles(players.len() as i32),
        play_area: Vec::new(),
        trash_pile: Vec::new(),
        players: players,
        pending_decision: None,
        pending_effects: vec![],
    };
}

pub fn run_game(players: &mut Vec<Box<Decider>>, debug: bool) -> Vec<f32> {
    let mut ctx = EvalContext {
        rng: randomly_seeded_weak_rng(),
        debug: debug,
    };

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
        println!();
    }

    return game.player_scores()
        .iter()
        .map(|&(_, score)| score)
        .collect();
}

#[cfg(test)]
mod tests {

    use game::*;
    use cards::*;

    fn advance_until_decision(game: &mut Game, ctx: &mut EvalContext) {
        while game.pending_decision.is_none() {
            game.advance_game(ctx);
        }
    }

    fn assert_decision(game: &mut Game, player_index: u8, decision_type: DecisionType) {
        if let Some(ref d) = game.pending_decision {
            assert_eq!(d.player, PlayerIdentifier(player_index));
            assert_eq!(d.decision_type, decision_type);
        } else {
            assert!(
                false,
                "No pending decision, expected for player {}, type {:?}",
                player_index, decision_type
            );
        }
    }

    #[test]
    fn test_player_targets() {
        let names = vec!["Player 1".into(), "Player 2".into(), "Player 3".into()];
        let g = fresh_game(&names);

        assert_eq!(
            g.players_for_target(EffectTarget::ActivePlayer, PlayerIdentifier(1))[0],
            PlayerIdentifier(1)
        );

        assert_eq!(
            g.players_for_target(EffectTarget::Opponents, PlayerIdentifier(1)),
            vec![PlayerIdentifier(2), PlayerIdentifier(0)]
        );

        assert_eq!(
            g.players_for_target(EffectTarget::AllPlayers, PlayerIdentifier(1)),
            vec![
                PlayerIdentifier(1),
                PlayerIdentifier(2),
                PlayerIdentifier(0),
            ]
        );
    }

    #[test]
    fn test_draw() {
        let mut ctx = EvalContext {
            debug: false,
            rng: randomly_seeded_weak_rng(),
        };
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

    #[test]
    fn test_militia_attack() {
        let names = vec!["Player 1".into(), "Player 2".into()];
        let mut ctx = EvalContext {
            debug: true,
            rng: randomly_seeded_weak_rng(),
        };
        let mut game = fresh_game(&names);

        game.players[0].hand.push(MILITIA.identifier);
        game.players[1].hand = vec![
            COPPER.identifier,
            COPPER.identifier,
            COPPER.identifier,
            COPPER.identifier,
            COPPER.identifier,
        ];

        advance_until_decision(&mut game, &mut ctx);
        game.resolve_decision(vec![MILITIA.identifier], &mut ctx);
        advance_until_decision(&mut game, &mut ctx);
        assert_decision(&mut game, 1, DecisionType::DiscardCards(None));

        game.resolve_decision(vec![COPPER.identifier, COPPER.identifier], &mut ctx);
        advance_until_decision(&mut game, &mut ctx);

        assert_decision(&mut game, 0, DecisionType::BuyCard);
        assert_eq!(game.players[1].hand.len(), 3);
    }

    #[test]
    fn test_moat_reveal() {
        let names = vec!["Player 1".into(), "Player 2".into()];
        let mut ctx = EvalContext {
            debug: true,
            rng: randomly_seeded_weak_rng(),
        };
        let mut game = fresh_game(&names);

        game.players[0].hand.push(MILITIA.identifier);
        game.players[1].hand = vec![
            MOAT.identifier,
            COPPER.identifier,
            COPPER.identifier,
            COPPER.identifier,
            COPPER.identifier,
        ];

        advance_until_decision(&mut game, &mut ctx);
        assert_decision(&mut game, 0, DecisionType::PlayAction);

        game.resolve_decision(vec![MILITIA.identifier], &mut ctx);
        advance_until_decision(&mut game, &mut ctx);
        let aid = game.current_action_identifier.clone();
        assert_decision(&mut game, 1, DecisionType::RevealReaction(aid));

        game.resolve_decision(vec![MOAT.identifier], &mut ctx);
        advance_until_decision(&mut game, &mut ctx);
        assert_decision(&mut game, 0, DecisionType::BuyCard);
        assert_eq!(game.players[1].hand.len(), 5);
    }

}

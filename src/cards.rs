use std;
use std::collections::HashMap;
use std::sync::Mutex;

#[allow(dead_code)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CardType {
    Treasure, Action, Victory, Reaction, Curse
}

#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct CardIdentifier(pub u16);

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GainDestination {
    GainToHand, GainToDiscard
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DiscardEffect {
    DrawPerDiscard
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TrashFollowup {
    ReplaceByCost(Option<CardType>, i32, GainDestination)
}

#[derive(Clone, Debug)]
pub enum CardAction {
    DiscardForEffect(DiscardEffect),
    DrawCards(i32),
    GainCardCostingUpto(i32),
    OpponentsDiscardTo(i32),
    PlusActions(i32),
    PlusBuys(i32),
    PlusCoins(i32),
    TrashCards(Option<CardType>, Option<TrashFollowup>)
}

#[derive(Clone, Debug)]
pub enum CardReaction {
    AttackImmunity
}

#[derive(Clone, Debug)]
pub enum EffectTarget {
     ActivePlayer,
     Opponents,
      #[allow(dead_code)]
     AllPlayers
}

pub fn target_for_action(action: &CardAction) -> EffectTarget {
    match action {
        &CardAction::OpponentsDiscardTo(_) => EffectTarget::Opponents,
        _ => EffectTarget::ActivePlayer
    }
}

#[derive(Debug)]
pub struct Card {
    pub identifier: CardIdentifier,
    pub name: &'static str,
    pub cost: i32,
    pub coin_value: Option<i32>,
    pub vp_value: Option<i32>,
    pub action_effects: Vec<CardAction>,
    pub reaction_effect: Option<CardReaction>,
    pub is_attack: bool
}

impl std::fmt::Display for CardIdentifier {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", lookup_card(self).name)
    }
}

impl Card {
    
    #[allow(dead_code)]
    pub fn is_action(&self) -> bool {
        self.action_effects.len() > 0
    }

    pub fn is_treasure(&self) -> bool {
        self.coin_value.is_some()
    }
    
    pub fn is_victory(&self) -> bool {
        self.vp_value.is_some()
    }
    
    pub fn is_reaction(&self) -> bool {
        self.reaction_effect.is_some()
    }

    #[allow(dead_code)]
    pub fn is_vp(&self) -> bool {
        match self.vp_value {
            Some(i) => i >= 0,
            None => false
        }
    }
    
    #[allow(dead_code)]
    pub fn is_curse(&self) -> bool {
        self.identifier == CURSE.identifier
    }
}

pub fn is_of_type(c: &CardIdentifier, card_type: &CardType) -> bool {
    let card = lookup_card(&c);
    match card_type {
        &CardType::Treasure => card.is_treasure(),
        &CardType::Action => card.is_action(),
        &CardType::Victory => card.is_victory(),
        &CardType::Reaction => card.is_reaction(),
        &CardType::Curse => card.is_curse(),
    }
}

pub fn filter_by_type(cards: &Vec<CardIdentifier>, card_type: &CardType) -> Vec<CardIdentifier> {
    cards.iter().filter(|c| is_of_type(c, card_type)).cloned().collect::<Vec<_>>()
}

impl std::fmt::Debug for CardIdentifier {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", lookup_card(self).name)
    }
}

lazy_static! {
    static ref CARD_ID_COUNTER : Mutex<u16> = Mutex::new(0);
}

fn bump_card_counter() -> u16 {
    let mut c = CARD_ID_COUNTER.lock().unwrap();
    *c += 1;
    c.clone()
}

fn make_treasure_card(name: &'static str, cost: i32, coin_value: i32) -> Card {
    Card {
        identifier: CardIdentifier(bump_card_counter()),
        name: name,
        cost: cost,
        coin_value: Some(coin_value),
        vp_value: None,
        action_effects: vec![],
        reaction_effect: None,
        is_attack: false
    }
}

fn make_vp_card(name: &'static str, cost: i32, vp_value: i32) -> Card {
    Card {
        identifier: CardIdentifier(bump_card_counter()),
        name: name,
        cost: cost,
        coin_value: None,
        vp_value: Some(vp_value),
        action_effects: vec![],
        reaction_effect: None,
        is_attack: false
    }
}

fn make_curse() -> Card {
    Card {
        identifier: CardIdentifier(bump_card_counter()),
        name: "Curse",
        cost: 0,
        coin_value: None,
        vp_value: Some(-1),
        action_effects: vec![],
        reaction_effect: None,
        is_attack: false
    }
}

fn make_action_card(name: &'static str, cost: i32, action_effects: Vec<CardAction>) -> Card {
    Card {
        identifier: CardIdentifier(bump_card_counter()),
        name: name,
        cost: cost,
        coin_value: None,
        vp_value: None,
        action_effects: action_effects,
        reaction_effect: None,
        is_attack: false
    }
}


fn make_attack_card(name: &'static str, cost: i32, action_effects: Vec<CardAction>) -> Card {
    Card {
        identifier: CardIdentifier(bump_card_counter()),
        name: name,
        cost: cost,
        coin_value: None,
        vp_value: None,
        action_effects: action_effects,
        reaction_effect: None,
        is_attack: true
    }
}

fn make_reaction_card(name: &'static str, cost: i32, action_effects: Vec<CardAction>, reaction: CardReaction) -> Card {
    Card {
        identifier: CardIdentifier(bump_card_counter()),
        name: name,
        cost: cost,
        coin_value: None,
        vp_value: None,
        action_effects: action_effects,
        reaction_effect: Some(reaction),
        is_attack: false
    }
}

// Ensure cards are correctly sorted by identifier, regardless of when lazy
// references are accessed.
// Can be replaced by const fns when available in stable, or custom macro.
fn sort_cards_by_identifier(v: Vec<&'static Card>) -> Vec<&'static Card> {
    let mut v = v;
    v.sort_by(|a, b| a.identifier.0.cmp(&b.identifier.0));
    v
}

fn trash_and_replace_action(card_type: Option<CardType>, plus_cost: i32, dest: GainDestination) -> CardAction {
    CardAction::TrashCards(card_type.clone(), Some(TrashFollowup::ReplaceByCost(card_type, plus_cost, dest)))
}

lazy_static! {
    
    pub static ref COPPER   : Card = make_treasure_card("Copper", 0, 1);
    pub static ref SILVER   : Card = make_treasure_card("Silver", 3, 2);
    pub static ref GOLD     : Card = make_treasure_card("Gold", 6, 3);
    pub static ref ESTATE   : Card = make_vp_card("Estate", 2, 1);
    pub static ref DUCHY    : Card = make_vp_card("Duchy", 5, 3);
    pub static ref PROVINCE : Card = make_vp_card("Province", 8, 6);
    pub static ref CURSE    : Card = make_curse();

    pub static ref VILLAGE : Card = make_action_card("Village", 3,
        vec![CardAction::DrawCards(1), CardAction::PlusActions(2)]);

    pub static ref SMITHY : Card = make_action_card("Smithy", 4,
        vec![CardAction::DrawCards(3)]);

    pub static ref WOODCUTTER : Card = make_action_card("Woodcutter", 3,
        vec![CardAction::PlusBuys(1), CardAction::PlusCoins(2)]);

    pub static ref MARKET : Card = make_action_card("Market", 5,
            vec![CardAction::DrawCards(1), CardAction::PlusActions(1),
                 CardAction::PlusBuys(1), CardAction::PlusCoins(1)]);

    pub static ref MILITIA : Card = make_attack_card("Militia", 4,
        vec![CardAction::PlusCoins(2), CardAction::OpponentsDiscardTo(3)]);
        
    pub static ref WORKSHOP : Card = make_action_card("Workshop", 3,
        vec![CardAction::GainCardCostingUpto(4)]);
        
    pub static ref MINE : Card = make_action_card("Mine", 5,
        vec![trash_and_replace_action(Some(CardType::Treasure), 3, GainDestination::GainToHand)]);

    pub static ref REMODEL : Card = make_action_card("Remodel", 5,
        vec![trash_and_replace_action(None, 2, GainDestination::GainToDiscard)]);
        
    pub static ref CELLAR : Card = make_action_card("Cellar", 2,
        vec![CardAction::DiscardForEffect(DiscardEffect::DrawPerDiscard)]);
        
    pub static ref MOAT : Card = make_reaction_card("Moat", 2,
        vec![CardAction::DrawCards(2)], CardReaction::AttackImmunity);

    pub static ref CARDS : Vec<&'static Card> = sort_cards_by_identifier(vec![
        &COPPER, &SILVER, &GOLD, &ESTATE, &DUCHY, &PROVINCE, &CURSE,
        &VILLAGE, &SMITHY, &MARKET, &WOODCUTTER, &MILITIA,
        &WORKSHOP, &MINE, &REMODEL, &CELLAR, &MOAT
    ]);    
}

pub fn lookup_card(ci: &CardIdentifier) -> &Card {
    return &CARDS[(ci.0 - 1) as usize];
}

pub fn card_names(identifiers: &Vec<CardIdentifier>) -> String {
    return identifiers.iter()
        .map(|ci| lookup_card(ci).name.to_string())
        .collect::<Vec<String>>().join(", ");
}

pub fn score_cards(identifiers: &Vec<CardIdentifier>) -> i32 {
    return identifiers.iter()
        .map(|ci| lookup_card(ci).vp_value.unwrap_or(0) )
        .fold(0, |sum, i| sum + i);
}

const VP_PILE_COUNT_2P: i32 = 8;
const VP_PILE_COUNT_MP: i32 = 12;
const KINGDOM_PILE_COUNT: i32 = 10;

pub fn standard_piles(num_players: i32) -> HashMap<CardIdentifier, i32> {
    let vp_count = if num_players == 2 { VP_PILE_COUNT_2P } else { VP_PILE_COUNT_MP };
    let curses = (num_players - 1) * 10;
    
    let mut cards = vec![(PROVINCE.identifier, vp_count),
         (DUCHY.identifier, vp_count),
         (ESTATE.identifier, vp_count),
         (GOLD.identifier, 30),
         (SILVER.identifier, 40),
         (COPPER.identifier, 46),
         (CURSE.identifier, curses)];
         
    let kingdom_cards = vec![
        VILLAGE.identifier, SMITHY.identifier, MARKET.identifier, WOODCUTTER.identifier, MILITIA.identifier,
        WORKSHOP.identifier, MINE.identifier, REMODEL.identifier, CELLAR.identifier, MOAT.identifier
    ];
    
    for c in kingdom_cards {
        cards.push((c, KINGDOM_PILE_COUNT));
    }
    
    cards.into_iter().collect::<HashMap<CardIdentifier, i32>>()
}

#[test]
fn test_card_identifiers() {
    for i in 0..CARDS.len() as i32 {
        let c1: &CardIdentifier = &CARDS[i as usize].identifier;
        let c2: CardIdentifier = lookup_card(&c1).identifier;
        assert_eq!(*c1, c2);
    }
}


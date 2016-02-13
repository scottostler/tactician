use std;
use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct CardIdentifier(pub u16);

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub enum GainDestination {
    GainIntoHand, GainToDiscard
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub enum DiscardEffect {
    DrawPerDiscard
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub enum CardAction {
    DiscardForEffect(DiscardEffect),
    DrawCards(i32),
    GainCardCostingUpto(i32),
    OpponentsDiscardTo(i32),
    PlusActions(i32),
    PlusBuys(i32),
    PlusCoins(i32),
}

#[derive(Debug)]
pub struct Card {
    pub identifier: CardIdentifier,
    pub name: &'static str,
    pub cost: i32,
    pub coin_value: Option<i32>,
    pub vp_value: Option<i32>,
    pub action_effects: Vec<CardAction>,
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
        action_effects: vec![]
    }
}

fn make_vp_card(name: &'static str, cost: i32, vp_value: i32) -> Card {
    Card {
        identifier: CardIdentifier(bump_card_counter()),
        name: name,
        cost: cost,
        coin_value: None,
        vp_value: Some(vp_value),
        action_effects: vec![]
    }
}

fn make_curse() -> Card {
    Card {
        identifier: CardIdentifier(bump_card_counter()),
        name: "Curse",
        cost: 0,
        coin_value: None,
        vp_value: Some(-1),
        action_effects: vec![]
    }
}

fn make_action_card(name: &'static str, cost: i32, action_effects: Vec<CardAction>) -> Card {
    Card {
        identifier: CardIdentifier(bump_card_counter()),
        name: name,
        cost: cost,
        coin_value: None,
        vp_value: None,
        action_effects: action_effects
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

lazy_static! {
    
    pub static ref COPPER   : Card = make_treasure_card("Copper", 0, 1);
    pub static ref SILVER   : Card = make_treasure_card("Silver", 3, 2);
    pub static ref GOLD     : Card = make_treasure_card("Gold", 6, 3);
    
    pub static ref ESTATE   : Card = make_vp_card("Estate", 2, 1);
    pub static ref DUCHY    : Card = make_vp_card("Duchy", 5, 3);
    pub static ref PROVINCE : Card = make_vp_card("Province", 8, 6);
    
    pub static ref CURSE    : Card = make_curse();

    pub static ref VILLAGE  : Card = make_action_card("Village", 3,
        vec![CardAction::DrawCards(1), CardAction::PlusActions(2)]);

    pub static ref SMITHY   : Card = make_action_card("Smithy", 4,
        vec![CardAction::DrawCards(3)]);

    pub static ref WOODCUTTER : Card = make_action_card("Woodcutter", 3,
        vec![CardAction::PlusBuys(1), CardAction::PlusCoins(2)]);

    pub static ref MARKET   : Card = make_action_card("Market", 5,
            vec![CardAction::DrawCards(1), CardAction::PlusActions(1), CardAction::PlusBuys(1), CardAction::PlusCoins(1)]);

    pub static ref CARDS : Vec<&'static Card> = sort_cards_by_identifier(vec![
        &COPPER, &SILVER, &GOLD, &ESTATE, &DUCHY, &PROVINCE, &CURSE,
        &VILLAGE, &SMITHY, &MARKET, &WOODCUTTER
    ]);
    
    // pub static ref KINGDOM_CARDS : Vec<&'static Card> = vec![
    //     &VILLAGE, &SMITHY
    // ];
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
    
    cards.push((VILLAGE.identifier, KINGDOM_PILE_COUNT));
    cards.push((SMITHY.identifier, KINGDOM_PILE_COUNT));
    cards.push((MARKET.identifier, KINGDOM_PILE_COUNT));
    cards.push((WOODCUTTER.identifier, KINGDOM_PILE_COUNT));
    
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


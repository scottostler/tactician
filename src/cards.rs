use std;
use std::collections::HashMap;

#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct CardIdentifier(pub i32);

pub struct Card {
    pub identifier: CardIdentifier,
    pub name: &'static str,
    pub cost: i32,
    pub coin_value: i32,
    pub vp_value: i32,
}

impl std::fmt::Display for CardIdentifier {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", lookup_card(self).name)
    }
}


impl Card {
    pub fn is_treasure(&self) -> bool {
        return self.coin_value > 0;
    }
}

pub const COPPER   : Card = Card { identifier: CardIdentifier(1), name: "Copper", cost: 0, coin_value: 1, vp_value: 0 };
pub const SILVER   : Card = Card { identifier: CardIdentifier(2), name: "Silver", cost: 3, coin_value: 2, vp_value: 0 };
pub const GOLD     : Card = Card { identifier: CardIdentifier(3), name: "Gold", cost: 6, coin_value: 3, vp_value: 0 };
pub const ESTATE   : Card = Card { identifier: CardIdentifier(4), name: "Estate", cost: 2, coin_value: 0, vp_value: 1 };
pub const DUCHY    : Card = Card { identifier: CardIdentifier(5), name: "Duchy", cost: 5, coin_value: 0, vp_value: 3 };
pub const PROVINCE : Card = Card { identifier: CardIdentifier(6), name: "Province", cost: 8, coin_value: 0, vp_value: 6 };
pub const CURSE    : Card = Card { identifier: CardIdentifier(7), name: "Curse", cost: 0, coin_value: 0, vp_value: -1 };

lazy_static! {
    static ref CARDS : Vec<Card> = vec![COPPER, SILVER, GOLD, ESTATE, DUCHY, PROVINCE, CURSE];
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
        .map(|ci| lookup_card(ci).vp_value)
        .fold(0, |sum, i| sum + i);
}

pub fn standard_piles(num_players: i32) -> HashMap<CardIdentifier, i32> {
    let vp_count = if num_players == 2 { 8 } else { 12 };
    let curses = (num_players - 1) * 10;
    return vec![(PROVINCE.identifier, vp_count),
         (DUCHY.identifier, vp_count),
         (ESTATE.identifier, vp_count),
         (GOLD.identifier, 30),
         (SILVER.identifier, 40),
         (COPPER.identifier, 46),
         (CURSE.identifier, curses)
    ].into_iter().collect();
}

pub type Card = u8;
pub type Combo = [Card; 2];

pub const DECK_SIZE: usize = 52;
pub const COMBO_COUNT: usize = 1326;

pub fn rank(card: Card) -> u8 {
    card / 4
}

pub fn suit(card: Card) -> u8 {
    card % 4
}

pub fn all_combos() -> Vec<Combo> {
    let mut out = Vec::with_capacity(COMBO_COUNT);
    for a in 0..DECK_SIZE as Card {
        for b in (a + 1)..DECK_SIZE as Card {
            out.push([a, b]);
        }
    }
    debug_assert_eq!(out.len(), COMBO_COUNT);
    out
}

pub fn disjoint(a: Combo, b: Combo) -> bool {
    a[0] != b[0] && a[0] != b[1] && a[1] != b[0] && a[1] != b[1]
}

pub fn class_key(combo: Combo) -> (u8, u8, u8) {
    let r0 = rank(combo[0]);
    let r1 = rank(combo[1]);
    let hi = r0.max(r1);
    let lo = r0.min(r1);
    let kind = if r0 == r1 { 0 } else if suit(combo[0]) == suit(combo[1]) { 1 } else { 2 };
    (hi, lo, kind)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn deck_has_52_unique_cards() {
        let cards: HashSet<Card> = (0..DECK_SIZE as Card).collect();
        assert_eq!(cards.len(), 52);
    }

    #[test]
    fn exact_combo_count_is_1326() {
        let combos = all_combos();
        assert_eq!(combos.len(), COMBO_COUNT);
        let unique: HashSet<(Card, Card)> = combos.iter().map(|c| (c[0], c[1])).collect();
        assert_eq!(unique.len(), COMBO_COUNT);
    }

    #[test]
    fn blockers_are_exact() {
        assert!(!disjoint([0, 4], [0, 8]));
        assert!(disjoint([0, 4], [8, 12]));
    }

    #[test]
    fn class_key_distinguishes_pair_suited_offsuit() {
        assert_eq!(class_key([48, 49]).2, 0); // AA pair
        assert_eq!(class_key([48, 44]).2, 1); // AK same suit
        assert_eq!(class_key([48, 45]).2, 2); // AK different suits
    }
}

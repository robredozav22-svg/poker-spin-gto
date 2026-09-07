use crate::cards::{rank, suit, Card};

pub type HandScore = [u8; 6];

fn straight_high(mut present: [bool; 13]) -> Option<u8> {
    // Ace also acts as rank below deuce for A2345.
    if present[12] && present[0] && present[1] && present[2] && present[3] {
        present[3] = true;
    }
    for high in (4u8..=12).rev() {
        let start = high - 4;
        if (start..=high).all(|r| present[r as usize]) {
            return Some(high);
        }
    }
    if present[12] && present[0] && present[1] && present[2] && present[3] {
        return Some(3); // five-high straight
    }
    None
}

pub fn evaluate_five(cards: [Card; 5]) -> HandScore {
    let mut counts = [0u8; 13];
    let mut present = [false; 13];
    let mut same_suit = true;
    let first_suit = suit(cards[0]);
    for c in cards {
        let r = rank(c) as usize;
        counts[r] += 1;
        present[r] = true;
        if suit(c) != first_suit {
            same_suit = false;
        }
    }

    if let Some(high) = straight_high(present) {
        if same_suit {
            return [8, high, 0, 0, 0, 0];
        }
    }

    let mut quads = None;
    let mut trips: Vec<u8> = Vec::new();
    let mut pairs: Vec<u8> = Vec::new();
    let mut singles: Vec<u8> = Vec::new();
    for r in (0u8..=12).rev() {
        match counts[r as usize] {
            4 => quads = Some(r),
            3 => trips.push(r),
            2 => pairs.push(r),
            1 => singles.push(r),
            _ => {}
        }
    }

    if let Some(q) = quads {
        return [7, q, singles[0], 0, 0, 0];
    }
    if !trips.is_empty() && (!pairs.is_empty() || trips.len() >= 2) {
        let pair_rank = if trips.len() >= 2 { trips[1] } else { pairs[0] };
        return [6, trips[0], pair_rank, 0, 0, 0];
    }
    if same_suit {
        let mut out = [5, 0, 0, 0, 0, 0];
        let mut k = 1;
        for r in (0u8..=12).rev() {
            if counts[r as usize] > 0 {
                out[k] = r;
                k += 1;
            }
        }
        return out;
    }
    if let Some(high) = straight_high(present) {
        return [4, high, 0, 0, 0, 0];
    }
    if !trips.is_empty() {
        return [3, trips[0], singles[0], singles[1], 0, 0];
    }
    if pairs.len() >= 2 {
        return [2, pairs[0], pairs[1], singles[0], 0, 0];
    }
    if pairs.len() == 1 {
        return [1, pairs[0], singles[0], singles[1], singles[2], 0];
    }
    let mut out = [0, 0, 0, 0, 0, 0];
    for (i, r) in singles.iter().take(5).enumerate() {
        out[i + 1] = *r;
    }
    out
}

pub fn evaluate_seven(cards: [Card; 7]) -> HandScore {
    let mut best = [0u8; 6];
    let mut initialized = false;
    for a in 0..3 {
        for b in (a + 1)..4 {
            for c in (b + 1)..5 {
                for d in (c + 1)..6 {
                    for e in (d + 1)..7 {
                        let score = evaluate_five([cards[a], cards[b], cards[c], cards[d], cards[e]]);
                        if !initialized || score > best {
                            best = score;
                            initialized = true;
                        }
                    }
                }
            }
        }
    }
    best
}

#[cfg(test)]
mod tests {
    use super::*;

    fn card(rank: u8, suit: u8) -> Card { rank * 4 + suit }

    #[test]
    fn straight_flush_beats_quads() {
        let sf = evaluate_five([card(12,0),card(11,0),card(10,0),card(9,0),card(8,0)]);
        let quads = evaluate_five([card(7,0),card(7,1),card(7,2),card(7,3),card(12,0)]);
        assert!(sf > quads);
    }

    #[test]
    fn wheel_is_five_high_straight() {
        let s = evaluate_five([card(12,0),card(0,1),card(1,2),card(2,3),card(3,0)]);
        assert_eq!(s[0],4);
        assert_eq!(s[1],3);
    }

    #[test]
    fn full_house_beats_flush() {
        let fh = evaluate_five([card(10,0),card(10,1),card(10,2),card(8,0),card(8,1)]);
        let fl = evaluate_five([card(12,0),card(9,0),card(7,0),card(4,0),card(1,0)]);
        assert!(fh > fl);
    }

    #[test]
    fn seven_cards_choose_best_five() {
        let score = evaluate_seven([card(12,0),card(12,1),card(12,2),card(5,0),card(5,1),card(2,0),card(1,1)]);
        assert_eq!(score[0],6);
        assert_eq!(score[1],12);
        assert_eq!(score[2],5);
    }
}

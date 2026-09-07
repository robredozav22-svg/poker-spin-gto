use crate::cards::{rank, suit, Card};

pub type HandScore = [u8; 6];

fn straight_high(mut present: [bool; 13]) -> Option<u8> {
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
        return Some(3);
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
        if suit(c) != first_suit { same_suit = false; }
    }

    if let Some(high) = straight_high(present) {
        if same_suit { return [8, high, 0, 0, 0, 0]; }
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

    if let Some(q) = quads { return [7, q, singles[0], 0, 0, 0]; }
    if !trips.is_empty() && (!pairs.is_empty() || trips.len() >= 2) {
        let pair_rank = if trips.len() >= 2 { trips[1] } else { pairs[0] };
        return [6, trips[0], pair_rank, 0, 0, 0];
    }
    if same_suit {
        let mut out = [5, 0, 0, 0, 0, 0];
        let mut k = 1;
        for r in (0u8..=12).rev() {
            if counts[r as usize] > 0 { out[k] = r; k += 1; }
        }
        return out;
    }
    if let Some(high) = straight_high(present) { return [4, high, 0, 0, 0, 0]; }
    if !trips.is_empty() { return [3, trips[0], singles[0], singles[1], 0, 0]; }
    if pairs.len() >= 2 { return [2, pairs[0], pairs[1], singles[0], 0, 0]; }
    if pairs.len() == 1 { return [1, pairs[0], singles[0], singles[1], singles[2], 0]; }
    let mut out = [0, 0, 0, 0, 0, 0];
    for (i, r) in singles.iter().take(5).enumerate() { out[i + 1] = *r; }
    out
}

/// Direct seven-card evaluator. Avoids enumerating all 21 five-card subsets,
/// which is critical for exact preflop board enumeration.
pub fn evaluate_seven(cards: [Card; 7]) -> HandScore {
    let mut counts=[0u8;13];
    let mut present=[false;13];
    let mut suit_counts=[0u8;4];
    let mut suit_present=[[false;13];4];
    for c in cards {
        let r=rank(c) as usize;
        let s=suit(c) as usize;
        counts[r]+=1;
        present[r]=true;
        suit_counts[s]+=1;
        suit_present[s][r]=true;
    }

    for s in 0..4 {
        if suit_counts[s]>=5 {
            if let Some(high)=straight_high(suit_present[s]) {
                return [8,high,0,0,0,0];
            }
        }
    }

    for r in (0u8..=12).rev() {
        if counts[r as usize]==4 {
            let kicker=(0u8..=12).rev().find(|k|*k!=r && counts[*k as usize]>0).unwrap();
            return [7,r,kicker,0,0,0];
        }
    }

    let top_trip=(0u8..=12).rev().find(|r|counts[*r as usize]>=3);
    if let Some(t)=top_trip {
        if let Some(p)=(0u8..=12).rev().find(|r|*r!=t && counts[*r as usize]>=2) {
            return [6,t,p,0,0,0];
        }
    }

    for s in 0..4 {
        if suit_counts[s]>=5 {
            let mut out=[5,0,0,0,0,0];
            let mut k=1usize;
            for r in (0u8..=12).rev() {
                if suit_present[s][r as usize] {
                    out[k]=r;k+=1;
                    if k==6 { break; }
                }
            }
            return out;
        }
    }

    if let Some(high)=straight_high(present) { return [4,high,0,0,0,0]; }

    if let Some(t)=top_trip {
        let mut out=[3,t,0,0,0,0];
        let mut k=2usize;
        for r in (0u8..=12).rev() {
            if r!=t && counts[r as usize]>0 {
                out[k]=r;k+=1;
                if k==4 { break; }
            }
        }
        return out;
    }

    let mut pair_ranks=[0u8;3];
    let mut pair_count=0usize;
    for r in (0u8..=12).rev() {
        if counts[r as usize]>=2 {
            if pair_count<3 { pair_ranks[pair_count]=r; }
            pair_count+=1;
        }
    }
    if pair_count>=2 {
        let p1=pair_ranks[0];let p2=pair_ranks[1];
        let kicker=(0u8..=12).rev().find(|r|*r!=p1 && *r!=p2 && counts[*r as usize]>0).unwrap();
        return [2,p1,p2,kicker,0,0];
    }
    if pair_count==1 {
        let p=pair_ranks[0];
        let mut out=[1,p,0,0,0,0];
        let mut k=2usize;
        for r in (0u8..=12).rev() {
            if r!=p && counts[r as usize]>0 {
                out[k]=r;k+=1;
                if k==5 { break; }
            }
        }
        return out;
    }

    let mut out=[0,0,0,0,0,0];
    let mut k=1usize;
    for r in (0u8..=12).rev() {
        if counts[r as usize]>0 {
            out[k]=r;k+=1;
            if k==6 { break; }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    fn card(rank: u8, suit: u8) -> Card { rank * 4 + suit }

    fn evaluate_seven_reference(cards:[Card;7])->HandScore{
        let mut best=[0u8;6];let mut initialized=false;
        for a in 0..3 { for b in (a+1)..4 { for c in (b+1)..5 { for d in (c+1)..6 { for e in (d+1)..7 {
            let score=evaluate_five([cards[a],cards[b],cards[c],cards[d],cards[e]]);
            if !initialized || score>best { best=score;initialized=true; }
        }}}}}
        best
    }

    #[test] fn straight_flush_beats_quads(){let sf=evaluate_five([card(12,0),card(11,0),card(10,0),card(9,0),card(8,0)]);let q=evaluate_five([card(7,0),card(7,1),card(7,2),card(7,3),card(12,0)]);assert!(sf>q);}
    #[test] fn wheel_is_five_high_straight(){let s=evaluate_five([card(12,0),card(0,1),card(1,2),card(2,3),card(3,0)]);assert_eq!(s[0],4);assert_eq!(s[1],3);}
    #[test] fn full_house_beats_flush(){let fh=evaluate_five([card(10,0),card(10,1),card(10,2),card(8,0),card(8,1)]);let fl=evaluate_five([card(12,0),card(9,0),card(7,0),card(4,0),card(1,0)]);assert!(fh>fl);}
    #[test] fn seven_cards_choose_best_five(){let s=evaluate_seven([card(12,0),card(12,1),card(12,2),card(5,0),card(5,1),card(2,0),card(1,1)]);assert_eq!(s[0],6);assert_eq!(s[1],12);assert_eq!(s[2],5);}

    #[test]
    fn direct_seven_matches_reference_on_deterministic_corpus(){
        // Walk 20k deterministic seven-card combinations spread through deck.
        let mut state=0x1234_5678_9ABC_DEF0u64;
        for _ in 0..20_000 {
            let mut deck=[0u8;52];for (i,c) in deck.iter_mut().enumerate(){*c=i as u8;}
            for i in 0..7 {
                state=state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
                let j=i+(state as usize%(52-i));deck.swap(i,j);
            }
            let cards=[deck[0],deck[1],deck[2],deck[3],deck[4],deck[5],deck[6]];
            assert_eq!(evaluate_seven(cards),evaluate_seven_reference(cards),"cards={cards:?}");
        }
    }
}

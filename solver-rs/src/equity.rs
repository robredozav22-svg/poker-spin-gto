use crate::cards::{rank, suit, Card, Combo, DECK_SIZE};
use crate::evaluator::evaluate_seven;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct HuMatchupKey(pub [Card; 4]);

fn mapped_card(card: Card, permutation: [u8; 4]) -> Card {
    rank(card) * 4 + permutation[suit(card) as usize]
}

fn permutations4() -> [[u8; 4]; 24] {
    let mut out = [[0u8; 4]; 24];
    let mut k = 0;
    for a in 0..4 {
        for b in 0..4 {
            if b == a { continue; }
            for c in 0..4 {
                if c == a || c == b { continue; }
                for d in 0..4 {
                    if d == a || d == b || d == c { continue; }
                    out[k] = [a,b,c,d];
                    k += 1;
                }
            }
        }
    }
    debug_assert_eq!(k,24);
    out
}

fn sorted_combo(mut combo: Combo) -> Combo {
    if combo[0] > combo[1] { combo.swap(0,1); }
    combo
}

pub fn canonical_hu_matchup(hero: Combo, villain: Combo) -> Result<HuMatchupKey,String> {
    if hero[0] == hero[1] || villain[0] == villain[1] {
        return Err("combo contains duplicate card".into());
    }
    if hero.iter().any(|h| villain.contains(h)) {
        return Err("hero/villain combos overlap".into());
    }
    let mut best: Option<[Card;4]> = None;
    for p in permutations4() {
        let h = sorted_combo([mapped_card(hero[0],p),mapped_card(hero[1],p)]);
        let v = sorted_combo([mapped_card(villain[0],p),mapped_card(villain[1],p)]);
        let candidate=[h[0],h[1],v[0],v[1]];
        if best.map(|b|candidate<b).unwrap_or(true) { best=Some(candidate); }
    }
    Ok(HuMatchupKey(best.expect("24 suit permutations")))
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EquityEstimate {
    pub hero: f64,
    pub villain: f64,
    pub ties: u64,
    pub samples: u64,
    pub seed: u64,
}

fn splitmix64(state: &mut u64) -> u64 {
    *state=state.wrapping_add(0x9E3779B97F4A7C15);
    let mut z=*state;
    z=(z^(z>>30)).wrapping_mul(0xBF58476D1CE4E5B9);
    z=(z^(z>>27)).wrapping_mul(0x94D049BB133111EB);
    z^(z>>31)
}

fn key_seed(key: HuMatchupKey, user_seed: u64) -> u64 {
    let mut x=user_seed ^ 0xA5A5_5A5A_D3C1_B79F;
    for card in key.0 { x=x.rotate_left(11) ^ (card as u64 + 0x9E37); }
    x
}

fn sample_board(state: &mut u64, dead: [Card;4]) -> [Card;5] {
    let mut deck=[0u8;DECK_SIZE];
    let mut n=0usize;
    for c in 0..DECK_SIZE as Card {
        if !dead.contains(&c) { deck[n]=c; n+=1; }
    }
    debug_assert_eq!(n,48);
    // Partial Fisher-Yates: choose five without replacement.
    for i in 0..5 {
        let remaining=n-i;
        let j=i+(splitmix64(state) as usize % remaining);
        deck.swap(i,j);
    }
    [deck[0],deck[1],deck[2],deck[3],deck[4]]
}

pub fn sampled_hu_equity(hero: Combo,villain: Combo,samples:u64,seed:u64)->Result<EquityEstimate,String>{
    if samples==0 { return Err("samples must be positive".into()); }
    let key=canonical_hu_matchup(hero,villain)?;
    let h=[key.0[0],key.0[1]];
    let v=[key.0[2],key.0[3]];
    let mut state=key_seed(key,seed);
    let mut hero_wins=0u64;
    let mut villain_wins=0u64;
    let mut ties=0u64;
    for _ in 0..samples {
        let b=sample_board(&mut state,[h[0],h[1],v[0],v[1]]);
        let hs=evaluate_seven([h[0],h[1],b[0],b[1],b[2],b[3],b[4]]);
        let vs=evaluate_seven([v[0],v[1],b[0],b[1],b[2],b[3],b[4]]);
        if hs>vs { hero_wins+=1; } else if vs>hs { villain_wins+=1; } else { ties+=1; }
    }
    let denom=samples as f64;
    Ok(EquityEstimate{
        hero:(hero_wins as f64 + 0.5*ties as f64)/denom,
        villain:(villain_wins as f64 + 0.5*ties as f64)/denom,
        ties,
        samples,
        seed,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn c(r:u8,s:u8)->Card{r*4+s}

    #[test]
    fn suit_renaming_has_same_canonical_key(){
        let a=canonical_hu_matchup([c(12,0),c(11,0)],[c(10,1),c(10,2)]).unwrap();
        let b=canonical_hu_matchup([c(12,3),c(11,3)],[c(10,0),c(10,1)]).unwrap();
        assert_eq!(a,b);
    }

    #[test]
    fn canonical_key_preserves_player_direction(){
        let h=[c(12,0),c(12,1)];
        let v=[c(11,2),c(11,3)];
        assert_ne!(canonical_hu_matchup(h,v).unwrap(),canonical_hu_matchup(v,h).unwrap());
    }

    #[test]
    fn sampled_equity_is_reproducible_and_zero_sum(){
        let h=[c(12,0),c(12,1)];
        let v=[c(11,2),c(11,3)];
        let a=sampled_hu_equity(h,v,2000,77).unwrap();
        let b=sampled_hu_equity(h,v,2000,77).unwrap();
        assert_eq!(a,b);
        assert!((a.hero+a.villain-1.0).abs()<1e-12);
    }

    #[test]
    fn suit_isomorphic_matchups_get_identical_sampled_result(){
        let a=sampled_hu_equity([c(12,0),c(11,0)],[c(10,1),c(10,2)],1500,99).unwrap();
        let b=sampled_hu_equity([c(12,3),c(11,3)],[c(10,0),c(10,1)],1500,99).unwrap();
        assert_eq!(a,b);
    }
}

use std::collections::HashMap;

use crate::cards::{Card,Combo,DECK_SIZE};
use crate::equity::{canonical_hu_matchup,HuMatchupKey};
use crate::evaluator::evaluate_seven;

pub const HU_PREFLOP_BOARD_COUNT:u64=1_712_304; // C(48,5)

#[derive(Debug,Clone,Copy,PartialEq)]
pub struct ExactEquity{
    pub hero:f64,
    pub villain:f64,
    pub wins:u64,
    pub losses:u64,
    pub ties:u64,
    pub boards:u64,
}

impl ExactEquity{
    pub fn zero_sum_error(self)->f64{(self.hero+self.villain-1.0).abs()}
}

fn remaining_deck(dead:[Card;4])->[Card;48]{
    let mut deck=[0u8;48];let mut n=0usize;
    for c in 0..DECK_SIZE as Card{
        if !dead.contains(&c){deck[n]=c;n+=1;}
    }
    assert_eq!(n,48);
    deck
}

pub fn exact_hu_equity(hero:Combo,villain:Combo)->Result<ExactEquity,String>{
    let key=canonical_hu_matchup(hero,villain)?;
    let h=[key.0[0],key.0[1]];
    let v=[key.0[2],key.0[3]];
    let deck=remaining_deck([h[0],h[1],v[0],v[1]]);
    let mut wins=0u64;let mut losses=0u64;let mut ties=0u64;let mut boards=0u64;

    for a in 0..44 {
        for b in (a+1)..45 {
            for c in (b+1)..46 {
                for d in (c+1)..47 {
                    for e in (d+1)..48 {
                        let board=[deck[a],deck[b],deck[c],deck[d],deck[e]];
                        let hs=evaluate_seven([h[0],h[1],board[0],board[1],board[2],board[3],board[4]]);
                        let vs=evaluate_seven([v[0],v[1],board[0],board[1],board[2],board[3],board[4]]);
                        if hs>vs{wins+=1;}else if vs>hs{losses+=1;}else{ties+=1;}
                        boards+=1;
                    }
                }
            }
        }
    }
    debug_assert_eq!(boards,HU_PREFLOP_BOARD_COUNT);
    let n=boards as f64;
    Ok(ExactEquity{
        hero:(wins as f64+0.5*ties as f64)/n,
        villain:(losses as f64+0.5*ties as f64)/n,
        wins,losses,ties,boards,
    })
}

#[derive(Debug,Default)]
pub struct ExactEquityCache{
    values:HashMap<HuMatchupKey,ExactEquity>,
    hits:u64,
    misses:u64,
}

impl ExactEquityCache{
    pub fn new()->Self{Self::default()}
    pub fn len(&self)->usize{self.values.len()}
    pub fn hits(&self)->u64{self.hits}
    pub fn misses(&self)->u64{self.misses}
    pub fn get_or_compute(&mut self,hero:Combo,villain:Combo)->Result<ExactEquity,String>{
        let key=canonical_hu_matchup(hero,villain)?;
        if let Some(v)=self.values.get(&key).copied(){self.hits+=1;return Ok(v);}
        let v=exact_hu_equity(hero,villain)?;
        self.values.insert(key,v);self.misses+=1;Ok(v)
    }
}

#[cfg(test)]
mod tests{
    use super::*;
    fn c(r:u8,s:u8)->Card{r*4+s}

    #[test]
    fn combinatorial_board_constant_is_correct(){
        let mut count=0u64;
        for a in 0..44{for b in (a+1)..45{for c in (b+1)..46{for d in (c+1)..47{for _e in (d+1)..48{count+=1;}}}}}
        assert_eq!(count,HU_PREFLOP_BOARD_COUNT);
    }

    #[test]
    fn exact_cache_uses_suit_canonical_key_without_computing_in_unit_test(){
        let a=canonical_hu_matchup([c(12,0),c(11,0)],[c(10,1),c(10,2)]).unwrap();
        let b=canonical_hu_matchup([c(12,3),c(11,3)],[c(10,0),c(10,1)]).unwrap();
        assert_eq!(a,b);
    }
}

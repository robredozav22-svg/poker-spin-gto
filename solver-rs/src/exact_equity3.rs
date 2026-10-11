use std::collections::HashMap;

use crate::cards::{Card,Combo,DECK_SIZE};
use crate::equity3::{canonical_threeway,ThreeWayKey};
use crate::evaluator::evaluate_seven;

pub const THREEWAY_PREFLOP_BOARD_COUNT:u64=1_370_754; // C(46,5)

#[derive(Debug,Clone,Copy,PartialEq)]
pub struct ExactThreeWayEquity{
    pub equities:[f64;3],
    pub outright_wins:[u64;3],
    pub two_way_ties:[u64;3],
    pub three_way_ties:u64,
    pub boards:u64,
}

impl ExactThreeWayEquity{
    pub fn zero_sum_error(self)->f64{(self.equities.iter().sum::<f64>()-1.0).abs()}
}

fn remaining_deck(dead:[Card;6])->[Card;46]{
    let mut deck=[0u8;46];let mut n=0usize;
    for card in 0..DECK_SIZE as Card{
        if !dead.contains(&card){deck[n]=card;n+=1;}
    }
    assert_eq!(n,46);deck
}

pub fn exact_threeway_equity(first:Combo,second:Combo,third:Combo)->Result<ExactThreeWayEquity,String>{
    let key=canonical_threeway(first,second,third)?;
    let hands=[[key.0[0],key.0[1]],[key.0[2],key.0[3]],[key.0[4],key.0[5]]];
    let deck=remaining_deck(key.0);
    let mut shares=[0.0f64;3];
    let mut outright=[0u64;3];
    // pair indexes: 0 => seats 0&1, 1 => 0&2, 2 => 1&2
    let mut two_way=[0u64;3];
    let mut three_way=0u64;
    let mut boards=0u64;

    for a in 0..42{
        for b in (a+1)..43{
            for c in (b+1)..44{
                for d in (c+1)..45{
                    for e in (d+1)..46{
                        let board=[deck[a],deck[b],deck[c],deck[d],deck[e]];
                        let scores=[
                            evaluate_seven([hands[0][0],hands[0][1],board[0],board[1],board[2],board[3],board[4]]),
                            evaluate_seven([hands[1][0],hands[1][1],board[0],board[1],board[2],board[3],board[4]]),
                            evaluate_seven([hands[2][0],hands[2][1],board[0],board[1],board[2],board[3],board[4]]),
                        ];
                        let best=*scores.iter().max().unwrap();
                        let w=[scores[0]==best,scores[1]==best,scores[2]==best];
                        let n=w.iter().filter(|x|**x).count();
                        match n{
                            1=>{let i=w.iter().position(|x|*x).unwrap();outright[i]+=1;shares[i]+=1.0;},
                            2=>{
                                let pair=match w{[true,true,false]=>0,[true,false,true]=>1,[false,true,true]=>2,_=>unreachable!()};
                                two_way[pair]+=1;for i in 0..3{if w[i]{shares[i]+=0.5;}}
                            },
                            3=>{three_way+=1;for s in &mut shares{*s+=1.0/3.0;}},
                            _=>unreachable!(),
                        }
                        boards+=1;
                    }
                }
            }
        }
    }
    debug_assert_eq!(boards,THREEWAY_PREFLOP_BOARD_COUNT);
    let n=boards as f64;
    Ok(ExactThreeWayEquity{equities:[shares[0]/n,shares[1]/n,shares[2]/n],outright_wins:outright,two_way_ties:two_way,three_way_ties:three_way,boards})
}

#[derive(Debug,Default)]
pub struct ExactThreeWayEquityCache{
    values:HashMap<ThreeWayKey,ExactThreeWayEquity>,
    hits:u64,
    misses:u64,
}

impl ExactThreeWayEquityCache{
    pub fn new()->Self{Self::default()}
    pub fn len(&self)->usize{self.values.len()}
    pub fn hits(&self)->u64{self.hits}
    pub fn misses(&self)->u64{self.misses}
    pub fn get_or_compute(&mut self,a:Combo,b:Combo,c:Combo)->Result<ExactThreeWayEquity,String>{
        let key=canonical_threeway(a,b,c)?;
        if let Some(v)=self.values.get(&key).copied(){self.hits+=1;return Ok(v);}
        let v=exact_threeway_equity(a,b,c)?;self.values.insert(key,v);self.misses+=1;Ok(v)
    }
}

#[cfg(test)]
mod tests{
    use super::*;
    use crate::cards::Card;
    fn c(r:u8,s:u8)->Card{r*4+s}

    #[test]
    fn board_count_constant_is_correct(){
        let mut count=0u64;
        for a in 0..42{for b in (a+1)..43{for c in (b+1)..44{for d in (c+1)..45{for _e in (d+1)..46{count+=1;}}}}}
        assert_eq!(count,THREEWAY_PREFLOP_BOARD_COUNT);
    }

    #[test]
    fn threeway_canonical_cache_key_preserves_roles(){
        let a=canonical_threeway([c(12,0),c(12,1)],[c(11,2),c(11,3)],[c(10,0),c(10,1)]).unwrap();
        let b=canonical_threeway([c(11,2),c(11,3)],[c(12,0),c(12,1)],[c(10,0),c(10,1)]).unwrap();
        assert_ne!(a,b);
    }
}

use std::collections::HashMap;

use crate::cards::Combo;
use crate::equity3::{canonical_threeway,sampled_threeway_equity,ThreeWayEquityEstimate,ThreeWayKey};

#[derive(Debug,Clone,Copy,PartialEq,Eq,Hash)]
struct CacheKey{
    matchup:ThreeWayKey,
    samples:u64,
    seed:u64,
}

#[derive(Debug,Default)]
pub struct ThreeWayEquityCache{
    values:HashMap<CacheKey,ThreeWayEquityEstimate>,
    hits:u64,
    misses:u64,
}

impl ThreeWayEquityCache{
    pub fn new()->Self{Self::default()}
    pub fn len(&self)->usize{self.values.len()}
    pub fn hits(&self)->u64{self.hits}
    pub fn misses(&self)->u64{self.misses}

    pub fn get_or_compute(
        &mut self,
        first:Combo,
        second:Combo,
        third:Combo,
        samples:u64,
        seed:u64,
    )->Result<ThreeWayEquityEstimate,String>{
        let matchup=canonical_threeway(first,second,third)?;
        let key=CacheKey{matchup,samples,seed};
        if let Some(value)=self.values.get(&key).copied(){
            self.hits+=1;
            return Ok(value);
        }
        let value=sampled_threeway_equity(first,second,third,samples,seed)?;
        self.values.insert(key,value);
        self.misses+=1;
        Ok(value)
    }
}

#[cfg(test)]
mod tests{
    use super::*;
    use crate::cards::Card;
    fn c(r:u8,s:u8)->Card{r*4+s}

    #[test]
    fn suit_isomorphic_threeway_reuses_cache(){
        let mut cache=ThreeWayEquityCache::new();
        let a=cache.get_or_compute(
            [c(12,0),c(11,0)],[c(10,1),c(10,2)],[c(9,1),c(8,1)],100,7
        ).unwrap();
        let b=cache.get_or_compute(
            [c(12,3),c(11,3)],[c(10,0),c(10,1)],[c(9,0),c(8,0)],100,7
        ).unwrap();
        assert_eq!(a,b);
        assert_eq!(cache.len(),1);
        assert_eq!(cache.misses(),1);
        assert_eq!(cache.hits(),1);
    }
}

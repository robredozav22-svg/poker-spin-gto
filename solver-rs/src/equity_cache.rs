use std::collections::HashMap;

use crate::cards::Combo;
use crate::equity::{canonical_hu_matchup, sampled_hu_equity, EquityEstimate, HuMatchupKey};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct CacheKey {
    matchup: HuMatchupKey,
    samples: u64,
    seed: u64,
}

#[derive(Debug, Default)]
pub struct EquityCache {
    values: HashMap<CacheKey, EquityEstimate>,
    hits: u64,
    misses: u64,
}

impl EquityCache {
    pub fn new() -> Self { Self::default() }

    pub fn get_or_compute(
        &mut self,
        hero: Combo,
        villain: Combo,
        samples: u64,
        seed: u64,
    ) -> Result<EquityEstimate,String> {
        let matchup=canonical_hu_matchup(hero,villain)?;
        let key=CacheKey{matchup,samples,seed};
        if let Some(value)=self.values.get(&key).copied(){
            self.hits+=1;
            return Ok(value);
        }
        let value=sampled_hu_equity(hero,villain,samples,seed)?;
        self.values.insert(key,value);
        self.misses+=1;
        Ok(value)
    }

    pub fn len(&self)->usize{self.values.len()}
    pub fn hits(&self)->u64{self.hits}
    pub fn misses(&self)->u64{self.misses}
}

#[cfg(test)]
mod tests{
    use super::*;
    use crate::cards::Card;
    fn c(r:u8,s:u8)->Card{r*4+s}

    #[test]
    fn suit_isomorphic_query_hits_same_cache_entry(){
        let mut cache=EquityCache::new();
        let a=cache.get_or_compute([c(12,0),c(11,0)],[c(10,1),c(10,2)],500,7).unwrap();
        let b=cache.get_or_compute([c(12,3),c(11,3)],[c(10,0),c(10,1)],500,7).unwrap();
        assert_eq!(a,b);
        assert_eq!(cache.len(),1);
        assert_eq!(cache.misses(),1);
        assert_eq!(cache.hits(),1);
    }

    #[test]
    fn sample_budget_and_seed_are_part_of_cache_key(){
        let mut cache=EquityCache::new();
        let h=[c(12,0),c(12,1)];
        let v=[c(11,2),c(11,3)];
        cache.get_or_compute(h,v,100,1).unwrap();
        cache.get_or_compute(h,v,200,1).unwrap();
        cache.get_or_compute(h,v,100,2).unwrap();
        assert_eq!(cache.len(),3);
        assert_eq!(cache.misses(),3);
    }
}

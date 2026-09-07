use crate::blockers::BlockerMatrix;
use crate::cards::{all_combos,Combo,COMBO_COUNT};
use crate::exact_equity::ExactEquityCache;
use crate::range::ComboRange;

#[derive(Debug,Clone,Copy,PartialEq)]
pub struct ExactRangeEquity{
    pub hero_equity:f64,
    pub compatible_weight:f64,
    pub compatible_combos:usize,
}

/// Exact HU preflop equity for one fixed hero combo versus a blocker-conditioned
/// opponent range. Every legal five-card board is enumerated for each canonical
/// hero/villain matchup and reused through ExactEquityCache.
pub fn exact_equity_vs_range(
    hero_combo_index:usize,
    villain_range:&ComboRange,
    blockers:&BlockerMatrix,
    cache:&mut ExactEquityCache,
)->Result<ExactRangeEquity,String>{
    if hero_combo_index>=COMBO_COUNT{return Err("hero combo index out of range".into());}
    let combos=all_combos();
    let hero:Combo=combos[hero_combo_index];
    let conditioned=villain_range.conditioned_on_blockers(hero_combo_index,blockers)?;
    let mut equity=0.0;let mut weight=0.0;let mut count=0usize;
    for villain_index in 0..COMBO_COUNT{
        let w=conditioned[villain_index];
        if w<=0.0{continue;}
        debug_assert!(blockers.compatible(hero_combo_index,villain_index));
        let e=cache.get_or_compute(hero,combos[villain_index])?;
        equity+=w*e.hero;
        weight+=w;
        count+=1;
    }
    if count==0 || weight<=f64::EPSILON{return Err("no blocker-compatible villain combos".into());}
    if (weight-1.0).abs()>1e-9{return Err(format!("conditioned villain weights sum to {weight}, expected 1"));}
    if !(0.0..=1.0).contains(&equity){return Err("integrated exact equity outside [0,1]".into());}
    Ok(ExactRangeEquity{hero_equity:equity,compatible_weight:weight,compatible_combos:count})
}

#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn fully_blocked_support_fails_before_exact_enumeration(){
        let blockers=BlockerMatrix::build();
        let hero=0usize;
        let mut w=vec![0.0;COMBO_COUNT];w[hero]=1.0;
        let villain=ComboRange::from_weights(w).unwrap();
        let mut cache=ExactEquityCache::new();
        assert!(exact_equity_vs_range(hero,&villain,&blockers,&mut cache).is_err());
        assert_eq!(cache.misses(),0);
    }
}

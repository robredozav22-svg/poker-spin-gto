use crate::blockers::BlockerMatrix;
use crate::cards::{all_combos, Combo, COMBO_COUNT};
use crate::equity_cache::EquityCache;
use crate::range::ComboRange;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RangeEquityEstimate {
    pub hero_equity: f64,
    pub compatible_weight: f64,
    pub compatible_combos: usize,
    pub samples_per_matchup: u64,
    pub seed: u64,
}

pub fn sampled_equity_vs_range(
    hero_combo_index: usize,
    villain_range: &ComboRange,
    blockers: &BlockerMatrix,
    cache: &mut EquityCache,
    samples_per_matchup: u64,
    seed: u64,
) -> Result<RangeEquityEstimate,String> {
    if hero_combo_index >= COMBO_COUNT { return Err("hero combo index out of range".into()); }
    if samples_per_matchup == 0 { return Err("samples_per_matchup must be positive".into()); }

    let combos=all_combos();
    let hero:Combo=combos[hero_combo_index];
    let conditioned=villain_range.conditioned_on_blockers(hero_combo_index,blockers)?;
    let mut equity=0.0;
    let mut count=0usize;
    let mut weight=0.0;

    for villain_index in 0..COMBO_COUNT {
        let w=conditioned[villain_index];
        if w<=0.0 { continue; }
        debug_assert!(blockers.compatible(hero_combo_index,villain_index));
        let estimate=cache.get_or_compute(hero,combos[villain_index],samples_per_matchup,seed)?;
        equity+=w*estimate.hero;
        weight+=w;
        count+=1;
    }

    Ok(RangeEquityEstimate{
        hero_equity:equity,
        compatible_weight:weight,
        compatible_combos:count,
        samples_per_matchup,
        seed,
    })
}

#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn uniform_range_uses_exactly_1225_blocker_compatible_combos(){
        let blockers=BlockerMatrix::build();
        let villain=ComboRange::uniform();
        let mut cache=EquityCache::new();
        let result=sampled_equity_vs_range(0,&villain,&blockers,&mut cache,4,11).unwrap();
        assert_eq!(result.compatible_combos,1225);
        assert!((result.compatible_weight-1.0).abs()<1e-12);
        assert!((0.0..=1.0).contains(&result.hero_equity));
    }

    #[test]
    fn repeated_query_reuses_cache(){
        let blockers=BlockerMatrix::build();
        let villain=ComboRange::uniform();
        let mut cache=EquityCache::new();
        sampled_equity_vs_range(0,&villain,&blockers,&mut cache,2,5).unwrap();
        let misses=cache.misses();
        sampled_equity_vs_range(0,&villain,&blockers,&mut cache,2,5).unwrap();
        assert_eq!(cache.misses(),misses);
        assert!(cache.hits()>0);
    }
}

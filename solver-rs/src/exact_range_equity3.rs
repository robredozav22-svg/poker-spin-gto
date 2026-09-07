use crate::blockers::BlockerMatrix;
use crate::cards::{all_combos,COMBO_COUNT};
use crate::exact_equity3::ExactThreeWayEquityCache;
use crate::range::ComboRange;

#[derive(Debug,Clone,Copy,PartialEq)]
pub struct ExactThreeWayRangeEquity{
    /// Equity order is [hero, first_range, second_range].
    pub equities:[f64;3],
    pub compatible_pair_weight:f64,
    pub compatible_pairs:usize,
}

/// Exact common-board three-way equity for one fixed hero combo versus two
/// independently supplied opponent priors, jointly conditioned on all card
/// removal. No pairwise-equity approximation and no board Monte Carlo.
pub fn exact_threeway_equity_vs_ranges(
    hero_combo_index:usize,
    first_range:&ComboRange,
    second_range:&ComboRange,
    blockers:&BlockerMatrix,
    cache:&mut ExactThreeWayEquityCache,
)->Result<ExactThreeWayRangeEquity,String>{
    if hero_combo_index>=COMBO_COUNT{return Err("hero combo index out of range".into());}
    let combos=all_combos();
    let hero=combos[hero_combo_index];
    let mut z=0.0;
    let mut out=[0.0;3];
    let mut count=0usize;

    for a in 0..COMBO_COUNT{
        let wa=first_range.weights()[a];
        if wa<=0.0 || !blockers.compatible(hero_combo_index,a){continue;}
        for b in 0..COMBO_COUNT{
            let wb=second_range.weights()[b];
            if wb<=0.0 || !blockers.compatible(hero_combo_index,b) || !blockers.compatible(a,b){continue;}
            let w=wa*wb;
            let e=cache.get_or_compute(hero,combos[a],combos[b])?;
            for seat in 0..3{out[seat]+=w*e.equities[seat];}
            z+=w;
            count+=1;
        }
    }
    if z<=f64::EPSILON{return Err("no jointly compatible opponent combo pairs".into());}
    for value in &mut out{*value/=z;}
    if (out.iter().sum::<f64>()-1.0).abs()>1e-9{return Err("exact integrated three-way equities do not sum to 1".into());}
    Ok(ExactThreeWayRangeEquity{equities:out,compatible_pair_weight:z,compatible_pairs:count})
}

#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn overlapping_ranges_fail_before_any_exact_board_enumeration(){
        let blockers=BlockerMatrix::build();
        let hero=0usize;
        let opponent=(0..COMBO_COUNT).find(|i|blockers.compatible(hero,*i)).unwrap();
        let mut w=vec![0.0;COMBO_COUNT];w[opponent]=1.0;
        let r=ComboRange::from_weights(w).unwrap();
        let mut cache=ExactThreeWayEquityCache::new();
        let err=exact_threeway_equity_vs_ranges(hero,&r,&r,&blockers,&mut cache).unwrap_err();
        assert!(err.contains("no jointly compatible"));
        assert_eq!(cache.misses(),0);
    }
}

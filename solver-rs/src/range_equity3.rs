use crate::blockers::BlockerMatrix;
use crate::cards::{all_combos,COMBO_COUNT};
use crate::equity3_cache::ThreeWayEquityCache;
use crate::range::ComboRange;

#[derive(Debug,Clone,Copy,PartialEq)]
pub struct ThreeWayRangeEquityEstimate{
    pub equities:[f64;3],
    pub compatible_pair_weight:f64,
    pub compatible_pairs:usize,
    pub samples_per_matchup:u64,
    pub seed:u64,
}

/// Hero is fixed as seat 0 in the returned equity vector. Opponent ranges are
/// integrated jointly, not independently: a pair contributes only when hero,
/// opponent A and opponent B are mutually card-disjoint.
pub fn sampled_threeway_equity_vs_ranges(
    hero_combo_index:usize,
    first_range:&ComboRange,
    second_range:&ComboRange,
    blockers:&BlockerMatrix,
    cache:&mut ThreeWayEquityCache,
    samples_per_matchup:u64,
    seed:u64,
)->Result<ThreeWayRangeEquityEstimate,String>{
    if hero_combo_index>=COMBO_COUNT{return Err("hero combo index out of range".into());}
    if samples_per_matchup==0{return Err("samples_per_matchup must be positive".into());}

    let combos=all_combos();
    let hero=combos[hero_combo_index];
    let first_positive:Vec<usize>=first_range.weights().iter().enumerate()
        .filter_map(|(i,w)|if *w>0.0 && blockers.compatible(hero_combo_index,i){Some(i)}else{None})
        .collect();
    let second_positive:Vec<usize>=second_range.weights().iter().enumerate()
        .filter_map(|(i,w)|if *w>0.0 && blockers.compatible(hero_combo_index,i){Some(i)}else{None})
        .collect();

    let mut z=0.0;
    let mut weighted=[0.0;3];
    let mut pairs=0usize;
    for &a in &first_positive{
        let wa=first_range.weights()[a];
        for &b in &second_positive{
            if !blockers.compatible(a,b){continue;}
            let wb=second_range.weights()[b];
            let w=wa*wb;
            if w<=0.0{continue;}
            let e=cache.get_or_compute(hero,combos[a],combos[b],samples_per_matchup,seed)?;
            for i in 0..3{weighted[i]+=w*e.equities[i];}
            z+=w;
            pairs+=1;
        }
    }
    if z<=f64::EPSILON{return Err("no jointly compatible opponent combo pairs".into());}
    for e in &mut weighted{*e/=z;}
    if (weighted.iter().sum::<f64>()-1.0).abs()>1e-9{
        return Err("integrated three-way equities do not sum to 1".into());
    }
    Ok(ThreeWayRangeEquityEstimate{
        equities:weighted,
        compatible_pair_weight:z,
        compatible_pairs:pairs,
        samples_per_matchup,
        seed,
    })
}

#[cfg(test)]
mod tests{
    use super::*;

    fn one_combo_range(index:usize)->ComboRange{
        let mut w=vec![0.0;COMBO_COUNT];
        w[index]=1.0;
        ComboRange::from_weights(w).unwrap()
    }

    #[test]
    fn sparse_joint_ranges_reduce_to_one_threeway_matchup(){
        let blockers=BlockerMatrix::build();
        let combos=all_combos();
        let hero_index=0usize;
        let mut a_index=None;
        let mut b_index=None;
        'outer:for a in 1..COMBO_COUNT{
            if !blockers.compatible(hero_index,a){continue;}
            for b in (a+1)..COMBO_COUNT{
                if blockers.compatible(hero_index,b)&&blockers.compatible(a,b){
                    a_index=Some(a);b_index=Some(b);break 'outer;
                }
            }
        }
        let a=a_index.unwrap();let b=b_index.unwrap();
        let ra=one_combo_range(a);let rb=one_combo_range(b);
        let mut cache=ThreeWayEquityCache::new();
        let out=sampled_threeway_equity_vs_ranges(hero_index,&ra,&rb,&blockers,&mut cache,100,9).unwrap();
        assert_eq!(out.compatible_pairs,1);
        assert!((out.compatible_pair_weight-1.0).abs()<1e-12);
        assert!((out.equities.iter().sum::<f64>()-1.0).abs()<1e-12);
        assert_eq!(cache.misses(),1);
        let direct=cache.get_or_compute(combos[hero_index],combos[a],combos[b],100,9).unwrap();
        assert_eq!(out.equities,direct.equities);
    }

    #[test]
    fn mutually_overlapping_opponent_ranges_fail_closed(){
        let blockers=BlockerMatrix::build();
        let hero_index=0usize;
        let a=(1..COMBO_COUNT).find(|i|blockers.compatible(hero_index,*i)).unwrap();
        let r=one_combo_range(a);
        let mut cache=ThreeWayEquityCache::new();
        let err=sampled_threeway_equity_vs_ranges(hero_index,&r,&r,&blockers,&mut cache,10,1).unwrap_err();
        assert!(err.contains("no jointly compatible"));
    }
}

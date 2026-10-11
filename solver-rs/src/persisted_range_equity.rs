use crate::blockers::BlockerMatrix;
use crate::cards::{all_combos,COMBO_COUNT};
use crate::exact_range_equity::ExactRangeEquity;
use crate::exact_range_equity3::ExactThreeWayRangeEquity;
use crate::payoff_lookup::{HuPayoffLookup,ThreeWayPayoffLookup};
use crate::range::ComboRange;

/// Exact HU range equity backed strictly by a persisted payoff lookup.
/// Every blocker-compatible opponent combo with positive conditioned mass must
/// exist in the table. Missing data fails closed; no enumeration or sampling
/// fallback occurs inside this runtime path.
pub fn persisted_equity_vs_range(
    hero_combo_index:usize,
    villain_range:&ComboRange,
    blockers:&BlockerMatrix,
    lookup:&HuPayoffLookup,
)->Result<ExactRangeEquity,String>{
    if hero_combo_index>=COMBO_COUNT{return Err("hero combo index out of range".into());}
    let combos=all_combos();
    let hero=combos[hero_combo_index];
    let conditioned=villain_range.conditioned_on_blockers(hero_combo_index,blockers)?;
    let mut equity=0.0;let mut weight=0.0;let mut count=0usize;
    for villain_index in 0..COMBO_COUNT{
        let w=conditioned[villain_index];
        if w<=0.0{continue;}
        let villain=combos[villain_index];
        let e=lookup.get_equity(hero,villain)?
            .ok_or_else(||format!("missing exact HU payoff key for hero={hero:?} villain={villain:?}"))?;
        equity+=w*e.hero;
        weight+=w;
        count+=1;
    }
    if count==0||weight<=f64::EPSILON{return Err("no blocker-compatible villain combos".into());}
    if (weight-1.0).abs()>1e-9{return Err(format!("conditioned villain weights sum to {weight}, expected 1"));}
    if !(0.0..=1.0).contains(&equity){return Err("persisted integrated exact equity outside [0,1]".into());}
    Ok(ExactRangeEquity{hero_equity:equity,compatible_weight:weight,compatible_combos:count})
}

/// Exact 3-way range equity backed strictly by persisted exact payoff records.
/// Equity order is [hero, first_range, second_range].
pub fn persisted_threeway_equity_vs_ranges(
    hero_combo_index:usize,
    first_range:&ComboRange,
    second_range:&ComboRange,
    blockers:&BlockerMatrix,
    lookup:&ThreeWayPayoffLookup,
)->Result<ExactThreeWayRangeEquity,String>{
    if hero_combo_index>=COMBO_COUNT{return Err("hero combo index out of range".into());}
    let combos=all_combos();
    let hero=combos[hero_combo_index];
    let mut z=0.0;let mut out=[0.0;3];let mut count=0usize;
    for a in 0..COMBO_COUNT{
        let wa=first_range.weights()[a];
        if wa<=0.0||!blockers.compatible(hero_combo_index,a){continue;}
        for b in 0..COMBO_COUNT{
            let wb=second_range.weights()[b];
            if wb<=0.0||!blockers.compatible(hero_combo_index,b)||!blockers.compatible(a,b){continue;}
            let first=combos[a];let second=combos[b];
            let e=lookup.get_equity(hero,first,second)?
                .ok_or_else(||format!("missing exact 3-way payoff key for hero={hero:?} first={first:?} second={second:?}"))?;
            let w=wa*wb;
            for seat in 0..3{out[seat]+=w*e.equities[seat];}
            z+=w;count+=1;
        }
    }
    if z<=f64::EPSILON{return Err("no jointly compatible opponent combo pairs".into());}
    for value in &mut out{*value/=z;}
    if (out.iter().sum::<f64>()-1.0).abs()>1e-9{return Err("persisted integrated three-way equities do not sum to 1".into());}
    Ok(ExactThreeWayRangeEquity{equities:out,compatible_pair_weight:z,compatible_pairs:count})
}

#[cfg(test)]
mod tests{
    use super::*;
    use crate::cards::{Card,Combo};
    use crate::payoff_table::{HuPayoffTable,ThreeWayPayoffTable};

    fn c(r:u8,s:u8)->Card{r*4+s}
    fn idx(target:Combo)->usize{all_combos().iter().position(|x|*x==target||*x==[target[1],target[0]]).unwrap()}
    fn singleton(i:usize)->ComboRange{let mut w=vec![0.0;COMBO_COUNT];w[i]=1.0;ComboRange::from_weights(w).unwrap()}

    #[test]
    fn hu_missing_required_key_fails_closed(){
        let hero=idx([c(12,0),c(12,1)]);let villain=idx([c(11,2),c(11,3)]);
        let blockers=BlockerMatrix::build();
        let lookup=HuPayoffLookup::from_table(HuPayoffTable{records:vec![]}).unwrap();
        let err=persisted_equity_vs_range(hero,&singleton(villain),&blockers,&lookup).unwrap_err();
        assert!(err.contains("missing exact HU payoff key"));
    }

    #[test]
    fn threeway_missing_required_key_fails_closed(){
        let hero=idx([c(12,0),c(12,1)]);let a=idx([c(11,2),c(11,3)]);let b=idx([c(10,0),c(10,1)]);
        let blockers=BlockerMatrix::build();
        let lookup=ThreeWayPayoffLookup::from_table(ThreeWayPayoffTable{records:vec![]}).unwrap();
        let err=persisted_threeway_equity_vs_ranges(hero,&singleton(a),&singleton(b),&blockers,&lookup).unwrap_err();
        assert!(err.contains("missing exact 3-way payoff key"));
    }
}

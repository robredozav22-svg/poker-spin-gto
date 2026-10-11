use std::collections::HashMap;

use crate::preflop_tree::PreflopAction;
use crate::strategy_record::{ComboActionFrequency,StrategyRecord};

#[derive(Debug,Clone,PartialEq)]
pub struct StrategyComparison{
    pub compared_combos:usize,
    pub mean_l1_per_combo:f64,
    pub max_abs_action_delta:f64,
    pub combos_with_any_delta_over_1pct:usize,
    pub combos_with_any_delta_over_5pct:usize,
}

/// Diagnostic comparison only. Frequency similarity is NOT an accuracy metric:
/// strategically indifferent actions can mix differently at nearly identical EV.
pub fn compare_physical_combo_frequencies(a:&StrategyRecord,b:&StrategyRecord)->Result<StrategyComparison,String>{
    if a.node!=b.node{return Err("cannot compare strategies from different node identities".into());}
    let a_map=index_rows(&a.combos)?;
    let b_map=index_rows(&b.combos)?;
    if a_map.len()!=b_map.len(){return Err("strategy records have different physical-combo coverage".into());}
    for key in a_map.keys(){if !b_map.contains_key(key){return Err(format!("strategy B missing combo index {key}"));}}

    let mut total_l1=0.0;
    let mut max_abs=0.0f64;
    let mut over1=0usize;
    let mut over5=0usize;

    for (combo,row_a) in &a_map{
        let row_b=b_map.get(combo).expect("coverage checked");
        let ma=action_map(row_a)?;
        let mb=action_map(row_b)?;
        let mut actions=std::collections::HashSet::new();
        actions.extend(ma.keys().copied());
        actions.extend(mb.keys().copied());
        let mut l1=0.0;
        let mut combo_max=0.0f64;
        for action in actions{
            let da=ma.get(&action).copied().unwrap_or(0.0);
            let db=mb.get(&action).copied().unwrap_or(0.0);
            let d=(da-db).abs();
            l1+=d;
            combo_max=combo_max.max(d);
            max_abs=max_abs.max(d);
        }
        total_l1+=l1;
        if combo_max>0.01{over1+=1;}
        if combo_max>0.05{over5+=1;}
    }

    let n=a_map.len();
    Ok(StrategyComparison{
        compared_combos:n,
        mean_l1_per_combo:if n==0{0.0}else{total_l1/n as f64},
        max_abs_action_delta:max_abs,
        combos_with_any_delta_over_1pct:over1,
        combos_with_any_delta_over_5pct:over5,
    })
}

fn index_rows(rows:&[ComboActionFrequency])->Result<HashMap<usize,&ComboActionFrequency>,String>{
    let mut out=HashMap::new();
    for row in rows{if out.insert(row.combo_index,row).is_some(){return Err(format!("duplicate combo index {}",row.combo_index));}}
    Ok(out)
}

fn action_map(row:&ComboActionFrequency)->Result<HashMap<PreflopAction,f64>,String>{
    let mut out=HashMap::new();
    for (a,p) in &row.frequencies{if out.insert(*a,*p).is_some(){return Err(format!("combo {} repeats action {:?}",row.combo_index,a));}}
    Ok(out)
}

#[cfg(test)]
mod tests{
    use super::*;
    use crate::preflop_tree::{Bb100,GameFormat,PayoutProfile,PreflopNodeKey};
    use crate::strategy_record::{StrategyRecord,StrategyVerification};
    use crate::tree::Player;

    fn record(rows:Vec<ComboActionFrequency>)->StrategyRecord{
        StrategyRecord{
            node:PreflopNodeKey::new(GameFormat::SpinHeadsUp,PayoutProfile::WinnerTakeAllChipEv,"fixture",Bb100(800),Player::Sb,vec![]).unwrap(),
            verification:StrategyVerification::Partial,
            source_id:"fixture".into(),
            solver_profile_id:"fixture".into(),
            combos:rows,
        }
    }

    #[test]
    fn identical_records_have_zero_frequency_distance(){
        let rows=vec![ComboActionFrequency{combo_index:0,frequencies:vec![(PreflopAction::Fold,0.25),(PreflopAction::JamTo(Bb100(800)),0.75)]}];
        let a=record(rows.clone());let b=record(rows);
        let c=compare_physical_combo_frequencies(&a,&b).unwrap();
        assert_eq!(c.mean_l1_per_combo,0.0);assert_eq!(c.max_abs_action_delta,0.0);
    }

    #[test]
    fn action_order_does_not_change_comparison(){
        let a=record(vec![ComboActionFrequency{combo_index:0,frequencies:vec![(PreflopAction::Fold,0.25),(PreflopAction::JamTo(Bb100(800)),0.75)]}]);
        let b=record(vec![ComboActionFrequency{combo_index:0,frequencies:vec![(PreflopAction::JamTo(Bb100(800)),0.75),(PreflopAction::Fold,0.25)]}]);
        assert_eq!(compare_physical_combo_frequencies(&a,&b).unwrap().mean_l1_per_combo,0.0);
    }

    #[test]
    fn large_mix_difference_is_reported_not_called_inaccurate(){
        let a=record(vec![ComboActionFrequency{combo_index:0,frequencies:vec![(PreflopAction::Fold,1.0)]}]);
        let b=record(vec![ComboActionFrequency{combo_index:0,frequencies:vec![(PreflopAction::JamTo(Bb100(800)),1.0)]}]);
        let c=compare_physical_combo_frequencies(&a,&b).unwrap();
        assert_eq!(c.mean_l1_per_combo,2.0);assert_eq!(c.max_abs_action_delta,1.0);assert_eq!(c.combos_with_any_delta_over_5pct,1);
    }

    #[test]
    fn different_nodes_cannot_be_compared(){
        let a=record(vec![]);let mut b=record(vec![]);b.node.effective_stack=Bb100(900);
        assert!(compare_physical_combo_frequencies(&a,&b).is_err());
    }
}

use crate::cards::COMBO_COUNT;
use crate::range::ComboRange;
use crate::StrategySnapshot;

fn validate_pair(a:&StrategySnapshot,b:&StrategySnapshot)->Result<(),String>{
    if a.infosets!=COMBO_COUNT||b.infosets!=COMBO_COUNT{return Err("strategy metrics require 1326 combo infosets".into());}
    if a.actions!=b.actions{return Err("strategy action counts differ".into());}
    if a.probabilities.len()!=a.infosets*a.actions||b.probabilities.len()!=b.infosets*b.actions{return Err("strategy snapshot shape mismatch".into());}
    Ok(())
}

pub fn max_action_delta_on_support(
    prior:&ComboRange,
    a:&StrategySnapshot,
    b:&StrategySnapshot,
)->Result<f64,String>{
    validate_pair(a,b)?;
    let mut max_delta=0.0f64;
    for i in 0..COMBO_COUNT{
        if prior.weights()[i]<=0.0{continue;}
        for action in 0..a.actions{
            max_delta=max_delta.max((a.row(i)[action]-b.row(i)[action]).abs());
        }
    }
    Ok(max_delta)
}

/// Prior-weighted mean absolute action-probability difference. The action
/// dimension is averaged within each combo, then combo errors are weighted by
/// the normalized private-hand prior.
pub fn weighted_action_mae(
    prior:&ComboRange,
    a:&StrategySnapshot,
    b:&StrategySnapshot,
)->Result<f64,String>{
    validate_pair(a,b)?;
    let mut total=0.0;
    for i in 0..COMBO_COUNT{
        let w=prior.weights()[i];if w<=0.0{continue;}
        let row_mae=(0..a.actions).map(|action|(a.row(i)[action]-b.row(i)[action]).abs()).sum::<f64>()/a.actions as f64;
        total+=w*row_mae;
    }
    Ok(total)
}

pub fn marginal_action_probability(
    prior:&ComboRange,
    strategy:&StrategySnapshot,
    action_index:usize,
)->Result<f64,String>{
    if strategy.infosets!=COMBO_COUNT{return Err("strategy metrics require 1326 combo infosets".into());}
    if action_index>=strategy.actions{return Err("action index out of range".into());}
    let mut p=0.0;
    for i in 0..COMBO_COUNT{p+=prior.weights()[i]*strategy.row(i)[action_index];}
    Ok(p)
}

#[cfg(test)]
mod tests{
    use super::*;

    fn one_combo_range(i:usize)->ComboRange{let mut w=vec![0.0;COMBO_COUNT];w[i]=1.0;ComboRange::from_weights(w).unwrap()}
    fn snapshot(i:usize,p_action1:f64)->StrategySnapshot{let mut p=vec![0.5;COMBO_COUNT*2];p[i*2]=1.0-p_action1;p[i*2+1]=p_action1;StrategySnapshot{infosets:COMBO_COUNT,actions:2,probabilities:p}}

    #[test]fn metrics_are_exact_on_one_combo_support(){let prior=one_combo_range(5);let a=snapshot(5,0.2);let b=snapshot(5,0.7);assert!((max_action_delta_on_support(&prior,&a,&b).unwrap()-0.5).abs()<1e-12);assert!((weighted_action_mae(&prior,&a,&b).unwrap()-0.5).abs()<1e-12);assert!((marginal_action_probability(&prior,&b,1).unwrap()-0.7).abs()<1e-12);}
    #[test]fn identical_snapshots_have_zero_delta(){let prior=ComboRange::uniform();let a=snapshot(0,0.4);assert_eq!(max_action_delta_on_support(&prior,&a,&a).unwrap(),0.0);assert_eq!(weighted_action_mae(&prior,&a,&a).unwrap(),0.0);}
}

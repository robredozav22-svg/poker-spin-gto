use crate::blockers::BlockerMatrix;
use crate::cards::COMBO_COUNT;
use crate::range::ComboRange;
use crate::StrategySnapshot;

#[derive(Debug,Clone,PartialEq)]
pub struct ConditionedActionRange{
    pub range:ComboRange,
    pub action_probability:f64,
}

#[derive(Debug,Clone,PartialEq)]
pub struct ActionMass{
    pub range:Option<ComboRange>,
    pub probability:f64,
}

fn validate_strategy(strategy:&StrategySnapshot,action_index:usize)->Result<(),String>{
    if strategy.infosets!=COMBO_COUNT{return Err(format!("expected {COMBO_COUNT} combo infosets, got {}",strategy.infosets));}
    if action_index>=strategy.actions{return Err("action index out of range".into());}
    if strategy.probabilities.len()!=strategy.infosets*strategy.actions{return Err("strategy snapshot shape mismatch".into());}
    Ok(())
}

pub fn condition_range_on_action(prior:&ComboRange,strategy:&StrategySnapshot,action_index:usize)->Result<ConditionedActionRange,String>{
    validate_strategy(strategy,action_index)?;
    let mut posterior_weights=vec![0.0;COMBO_COUNT];let mut p=0.0;
    for i in 0..COMBO_COUNT{
        let a=strategy.row(i)[action_index];
        if !a.is_finite()||a<0.0||a>1.0{return Err(format!("invalid action probability at combo {i}"));}
        let joint=prior.weights()[i]*a;posterior_weights[i]=joint;p+=joint;
    }
    if p<=f64::EPSILON{return Err("chosen action has zero probability under prior".into());}
    Ok(ConditionedActionRange{range:ComboRange::from_weights(posterior_weights)?,action_probability:p})
}

/// Returns P(action | hero cards) and, when nonzero, the posterior
/// P(opponent combo | action, hero cards). Zero-probability actions are valid
/// and return `range=None` rather than causing the solver to fail.
pub fn action_mass_on_hero_blockers(
    hero_combo_index:usize,
    opponent_prior:&ComboRange,
    opponent_strategy:&StrategySnapshot,
    action_index:usize,
    blockers:&BlockerMatrix,
)->Result<ActionMass,String>{
    if hero_combo_index>=COMBO_COUNT{return Err("hero combo index out of range".into());}
    validate_strategy(opponent_strategy,action_index)?;
    let conditioned=opponent_prior.conditioned_on_blockers(hero_combo_index,blockers)?;
    let mut weights=vec![0.0;COMBO_COUNT];let mut p=0.0;
    for i in 0..COMBO_COUNT{
        let a=opponent_strategy.row(i)[action_index];
        if !a.is_finite()||a<0.0||a>1.0{return Err(format!("invalid action probability at combo {i}"));}
        let joint=conditioned[i]*a;weights[i]=joint;p+=joint;
    }
    let range=if p<=f64::EPSILON{None}else{Some(ComboRange::from_weights(weights)?)};
    Ok(ActionMass{range,probability:p})
}

pub fn condition_action_on_hero_blockers(
    hero_combo_index:usize,
    opponent_prior:&ComboRange,
    opponent_strategy:&StrategySnapshot,
    action_index:usize,
    blockers:&BlockerMatrix,
)->Result<ConditionedActionRange,String>{
    let out=action_mass_on_hero_blockers(hero_combo_index,opponent_prior,opponent_strategy,action_index,blockers)?;
    match out.range{
        Some(range)=>Ok(ConditionedActionRange{range,action_probability:out.probability}),
        None=>Err("chosen action has zero probability after hero blockers".into()),
    }
}

#[cfg(test)]
mod tests{
    use super::*;
    fn snapshot_with_action1(prob:impl Fn(usize)->f64)->StrategySnapshot{
        let mut probabilities=Vec::with_capacity(COMBO_COUNT*2);
        for i in 0..COMBO_COUNT{let p=prob(i);probabilities.push(1.0-p);probabilities.push(p);}
        StrategySnapshot{infosets:COMBO_COUNT,actions:2,probabilities}
    }

    #[test]fn always_action_preserves_prior(){let prior=ComboRange::uniform();let s=snapshot_with_action1(|_|1.0);let out=condition_range_on_action(&prior,&s,1).unwrap();assert!((out.action_probability-1.0).abs()<1e-12);for i in [0usize,100,900,1325]{assert!((out.range.weights()[i]-prior.weights()[i]).abs()<1e-12);}}
    #[test]fn posterior_is_weighted_by_action_frequency(){let mut w=vec![0.0;COMBO_COUNT];w[10]=0.5;w[20]=0.5;let prior=ComboRange::from_weights(w).unwrap();let s=snapshot_with_action1(|i|if i==10{1.0}else if i==20{0.25}else{0.0});let out=condition_range_on_action(&prior,&s,1).unwrap();assert!((out.action_probability-0.625).abs()<1e-12);assert!((out.range.weights()[10]-0.8).abs()<1e-12);assert!((out.range.weights()[20]-0.2).abs()<1e-12);}
    #[test]fn hero_blockers_remove_overlaps_before_action_probability(){let blockers=BlockerMatrix::build();let hero=0usize;let compatible=(0..COMBO_COUNT).find(|i|*i!=hero&&blockers.compatible(hero,*i)).unwrap();let blocked=(0..COMBO_COUNT).find(|i|*i!=hero&&!blockers.compatible(hero,*i)).unwrap();let mut w=vec![0.0;COMBO_COUNT];w[compatible]=0.5;w[blocked]=0.5;let prior=ComboRange::from_weights(w).unwrap();let s=snapshot_with_action1(|i|if i==compatible{0.25}else if i==blocked{1.0}else{0.0});let out=condition_action_on_hero_blockers(hero,&prior,&s,1,&blockers).unwrap();assert!((out.action_probability-0.25).abs()<1e-12);assert!((out.range.weights()[compatible]-1.0).abs()<1e-12);assert_eq!(out.range.weights()[blocked],0.0);}
    #[test]fn zero_probability_action_is_valid_mass_without_range(){let blockers=BlockerMatrix::build();let prior=ComboRange::uniform();let s=snapshot_with_action1(|_|0.0);let out=action_mass_on_hero_blockers(0,&prior,&s,1,&blockers).unwrap();assert_eq!(out.probability,0.0);assert!(out.range.is_none());}
    #[test]fn legacy_conditioner_still_fails_closed_on_impossible_action(){let prior=ComboRange::uniform();let s=snapshot_with_action1(|_|0.0);assert!(condition_range_on_action(&prior,&s,1).is_err());}
}

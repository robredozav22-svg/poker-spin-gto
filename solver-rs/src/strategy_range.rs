use crate::blockers::BlockerMatrix;
use crate::cards::COMBO_COUNT;
use crate::range::ComboRange;
use crate::StrategySnapshot;

#[derive(Debug,Clone,PartialEq)]
pub struct ConditionedActionRange{
    pub range:ComboRange,
    pub action_probability:f64,
}

fn validate_strategy(strategy:&StrategySnapshot,action_index:usize)->Result<(),String>{
    if strategy.infosets!=COMBO_COUNT {
        return Err(format!("expected {COMBO_COUNT} combo infosets, got {}",strategy.infosets));
    }
    if action_index>=strategy.actions {
        return Err("action index out of range".into());
    }
    if strategy.probabilities.len()!=strategy.infosets*strategy.actions {
        return Err("strategy snapshot shape mismatch".into());
    }
    Ok(())
}

/// Convert a prior combo distribution plus a per-combo mixed strategy into
/// P(combo | chosen action). The returned action_probability is the marginal
/// P(action) under the supplied prior.
pub fn condition_range_on_action(
    prior:&ComboRange,
    strategy:&StrategySnapshot,
    action_index:usize,
)->Result<ConditionedActionRange,String>{
    validate_strategy(strategy,action_index)?;

    let mut posterior_weights=vec![0.0;COMBO_COUNT];
    let mut action_probability=0.0;
    for i in 0..COMBO_COUNT {
        let p_action=strategy.row(i)[action_index];
        if !p_action.is_finite() || p_action<0.0 || p_action>1.0 {
            return Err(format!("invalid action probability at combo {i}"));
        }
        let joint=prior.weights()[i]*p_action;
        posterior_weights[i]=joint;
        action_probability+=joint;
    }
    if action_probability<=f64::EPSILON {
        return Err("chosen action has zero probability under prior".into());
    }
    let range=ComboRange::from_weights(posterior_weights)?;
    Ok(ConditionedActionRange{range,action_probability})
}

/// Exact action posterior from the perspective of a fixed hero combo.
///
/// First conditions the opponent prior on card removal from `hero_combo_index`,
/// then applies the opponent's per-combo mixed strategy. Therefore
/// `action_probability` is P(action | hero cards), and `range` is
/// P(opponent combo | action, hero cards).
pub fn condition_action_on_hero_blockers(
    hero_combo_index:usize,
    opponent_prior:&ComboRange,
    opponent_strategy:&StrategySnapshot,
    action_index:usize,
    blockers:&BlockerMatrix,
)->Result<ConditionedActionRange,String>{
    if hero_combo_index>=COMBO_COUNT{return Err("hero combo index out of range".into());}
    validate_strategy(opponent_strategy,action_index)?;

    let blocker_conditioned=opponent_prior.conditioned_on_blockers(hero_combo_index,blockers)?;
    let mut posterior_weights=vec![0.0;COMBO_COUNT];
    let mut action_probability=0.0;
    for i in 0..COMBO_COUNT {
        let p_action=opponent_strategy.row(i)[action_index];
        if !p_action.is_finite() || p_action<0.0 || p_action>1.0 {
            return Err(format!("invalid action probability at combo {i}"));
        }
        let joint=blocker_conditioned[i]*p_action;
        posterior_weights[i]=joint;
        action_probability+=joint;
    }
    if action_probability<=f64::EPSILON {
        return Err("chosen action has zero probability after hero blockers".into());
    }
    let range=ComboRange::from_weights(posterior_weights)?;
    Ok(ConditionedActionRange{range,action_probability})
}

#[cfg(test)]
mod tests{
    use super::*;

    fn snapshot_with_action1(prob:impl Fn(usize)->f64)->StrategySnapshot{
        let mut probabilities=Vec::with_capacity(COMBO_COUNT*2);
        for i in 0..COMBO_COUNT {
            let p=prob(i);
            probabilities.push(1.0-p);
            probabilities.push(p);
        }
        StrategySnapshot{infosets:COMBO_COUNT,actions:2,probabilities}
    }

    #[test]
    fn always_action_preserves_prior(){
        let prior=ComboRange::uniform();
        let s=snapshot_with_action1(|_|1.0);
        let out=condition_range_on_action(&prior,&s,1).unwrap();
        assert!((out.action_probability-1.0).abs()<1e-12);
        for i in [0usize,100,900,1325] {
            assert!((out.range.weights()[i]-prior.weights()[i]).abs()<1e-12);
        }
    }

    #[test]
    fn posterior_is_weighted_by_action_frequency(){
        let mut w=vec![0.0;COMBO_COUNT];
        w[10]=0.5;
        w[20]=0.5;
        let prior=ComboRange::from_weights(w).unwrap();
        let s=snapshot_with_action1(|i|if i==10{1.0}else if i==20{0.25}else{0.0});
        let out=condition_range_on_action(&prior,&s,1).unwrap();
        assert!((out.action_probability-0.625).abs()<1e-12);
        assert!((out.range.weights()[10]-0.8).abs()<1e-12);
        assert!((out.range.weights()[20]-0.2).abs()<1e-12);
    }

    #[test]
    fn hero_blockers_remove_overlapping_opponent_combos_before_action_probability(){
        let blockers=BlockerMatrix::build();
        let hero=0usize;
        let mut compatible=None;
        let mut blocked=None;
        for i in 0..COMBO_COUNT{
            if i==hero{continue;}
            if blockers.compatible(hero,i)&&compatible.is_none(){compatible=Some(i);}
            if !blockers.compatible(hero,i)&&blocked.is_none(){blocked=Some(i);}
            if compatible.is_some()&&blocked.is_some(){break;}
        }
        let compatible=compatible.unwrap();
        let blocked=blocked.unwrap();
        let mut w=vec![0.0;COMBO_COUNT];
        w[compatible]=0.5;
        w[blocked]=0.5;
        let prior=ComboRange::from_weights(w).unwrap();
        let s=snapshot_with_action1(|i|if i==compatible{0.25}else if i==blocked{1.0}else{0.0});
        let out=condition_action_on_hero_blockers(hero,&prior,&s,1,&blockers).unwrap();
        // Blocked combo disappears before the action posterior. Only the
        // compatible combo remains in the prior, so P(action|hero)=0.25.
        assert!((out.action_probability-0.25).abs()<1e-12);
        assert!((out.range.weights()[compatible]-1.0).abs()<1e-12);
        assert_eq!(out.range.weights()[blocked],0.0);
    }

    #[test]
    fn impossible_action_fails_closed(){
        let prior=ComboRange::uniform();
        let s=snapshot_with_action1(|_|0.0);
        assert!(condition_range_on_action(&prior,&s,1).is_err());
    }
}

use crate::cards::COMBO_COUNT;
use crate::range::ComboRange;
use crate::StrategySnapshot;

#[derive(Debug,Clone,PartialEq)]
pub struct ConditionedActionRange{
    pub range:ComboRange,
    pub action_probability:f64,
}

/// Convert a prior combo distribution plus a per-combo mixed strategy into
/// P(combo | chosen action). The returned action_probability is the marginal
/// P(action) under the supplied prior.
pub fn condition_range_on_action(
    prior:&ComboRange,
    strategy:&StrategySnapshot,
    action_index:usize,
)->Result<ConditionedActionRange,String>{
    if strategy.infosets!=COMBO_COUNT {
        return Err(format!("expected {COMBO_COUNT} combo infosets, got {}",strategy.infosets));
    }
    if action_index>=strategy.actions {
        return Err("action index out of range".into());
    }
    if strategy.probabilities.len()!=strategy.infosets*strategy.actions {
        return Err("strategy snapshot shape mismatch".into());
    }

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
    fn impossible_action_fails_closed(){
        let prior=ComboRange::uniform();
        let s=snapshot_with_action1(|_|0.0);
        assert!(condition_range_on_action(&prior,&s,1).is_err());
    }
}

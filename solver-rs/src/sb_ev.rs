use crate::blockers::BlockerMatrix;
use crate::cards::COMBO_COUNT;
use crate::equity_cache::EquityCache;
use crate::range::ComboRange;
use crate::range_equity::{sampled_equity_vs_range,RangeEquityEstimate};
use crate::strategy_range::action_mass_on_hero_blockers;
use crate::terminal::{Seat,TerminalId,TerminalPot};
use crate::terminal_ev::expected_two_active_payoff;
use crate::StrategySnapshot;

pub const BB_FOLD_ACTION:usize=0;
pub const BB_CALL_ACTION:usize=1;

#[derive(Debug,Clone,Copy,PartialEq)]
pub struct SbFoldJamValues{
    pub fold_ev_sb:f64,
    pub jam_ev_sb:f64,
    pub bb_fold_probability:f64,
    pub bb_call_probability:f64,
    pub equity_when_called:Option<RangeEquityEstimate>,
}

impl SbFoldJamValues{
    pub fn action_values(self)->[f64;2]{[self.fold_ev_sb,self.jam_ev_sb]}
}

/// SB decision after BTN folds in the restricted push/fold tree.
///
/// BB response is evaluated conditional on the exact SB hole cards. Thus both
/// P(BB folds/calls) and the BB call range include card removal before action
/// selection. Action order in `bb_strategy` is [Fold, Call].
pub fn sampled_sb_fold_jam_values(
    stack_bb:f64,
    sb_combo_index:usize,
    bb_prior:&ComboRange,
    bb_strategy:&StrategySnapshot,
    blockers:&BlockerMatrix,
    cache:&mut EquityCache,
    samples_per_matchup:u64,
    seed:u64,
)->Result<SbFoldJamValues,String>{
    if sb_combo_index>=COMBO_COUNT{return Err("SB combo index out of range".into());}
    if bb_strategy.actions!=2{return Err("BB response strategy must have [Fold, Call] actions".into());}

    let fold_mass=action_mass_on_hero_blockers(sb_combo_index,bb_prior,bb_strategy,BB_FOLD_ACTION,blockers)?;
    let call_mass=action_mass_on_hero_blockers(sb_combo_index,bb_prior,bb_strategy,BB_CALL_ACTION,blockers)?;
    let total=fold_mass.probability+call_mass.probability;
    if (total-1.0).abs()>1e-9{return Err(format!("BB action probabilities after blockers sum to {total}, expected 1"));}

    let fold_terminal=TerminalPot::for_terminal(TerminalId::BtnFoldSbFold,stack_bb);
    let fold_ev=fold_terminal.settle(&[Seat::Bb])[Seat::Sb as usize];

    let bb_folds_terminal=TerminalPot::for_terminal(TerminalId::BtnFoldSbJamBbFold,stack_bb);
    let jam_when_fold=bb_folds_terminal.settle(&[Seat::Sb])[Seat::Sb as usize];

    let mut jam_ev=fold_mass.probability*jam_when_fold;
    let mut equity_when_called=None;
    if call_mass.probability>f64::EPSILON{
        let call_range=call_mass.range.as_ref().ok_or("positive BB call probability without call range")?;
        let eq=sampled_equity_vs_range(sb_combo_index,call_range,blockers,cache,samples_per_matchup,seed)?;
        let call_terminal=TerminalPot::for_terminal(TerminalId::BtnFoldSbJamBbCall,stack_bb);
        let showdown=expected_two_active_payoff(&call_terminal,Seat::Sb,Seat::Bb,eq.hero_equity);
        jam_ev+=call_mass.probability*showdown[Seat::Sb as usize];
        equity_when_called=Some(eq);
    }

    Ok(SbFoldJamValues{
        fold_ev_sb:fold_ev,
        jam_ev_sb:jam_ev,
        bb_fold_probability:fold_mass.probability,
        bb_call_probability:call_mass.probability,
        equity_when_called,
    })
}

#[cfg(test)]
mod tests{
    use super::*;

    fn pure_bb_strategy(call_combo:Option<usize>)->StrategySnapshot{
        let mut p=Vec::with_capacity(COMBO_COUNT*2);
        for i in 0..COMBO_COUNT{
            let call=if call_combo==Some(i){1.0}else{0.0};
            p.push(1.0-call);p.push(call);
        }
        StrategySnapshot{infosets:COMBO_COUNT,actions:2,probabilities:p}
    }

    #[test]
    fn if_bb_always_folds_sb_jam_ev_is_plus_one(){
        let blockers=BlockerMatrix::build();let prior=ComboRange::uniform();let s=pure_bb_strategy(None);let mut cache=EquityCache::new();
        let v=sampled_sb_fold_jam_values(8.0,0,&prior,&s,&blockers,&mut cache,1,1).unwrap();
        assert_eq!(v.fold_ev_sb,-0.5);assert!((v.bb_fold_probability-1.0).abs()<1e-12);assert_eq!(v.bb_call_probability,0.0);assert!((v.jam_ev_sb-1.0).abs()<1e-12);assert!(v.equity_when_called.is_none());
    }

    #[test]
    fn one_compatible_call_combo_produces_finite_called_equity(){
        let blockers=BlockerMatrix::build();let sb=0usize;let call=(0..COMBO_COUNT).find(|i|blockers.compatible(sb,*i)).unwrap();
        let mut w=vec![0.0;COMBO_COUNT];w[call]=1.0;let prior=ComboRange::from_weights(w).unwrap();let s=pure_bb_strategy(Some(call));let mut cache=EquityCache::new();
        let v=sampled_sb_fold_jam_values(8.0,sb,&prior,&s,&blockers,&mut cache,20,7).unwrap();
        assert_eq!(v.bb_fold_probability,0.0);assert_eq!(v.bb_call_probability,1.0);let eq=v.equity_when_called.unwrap();assert_eq!(eq.compatible_combos,1);assert!((v.jam_ev_sb-(16.0*eq.hero_equity-8.0)).abs()<1e-12);
    }
}

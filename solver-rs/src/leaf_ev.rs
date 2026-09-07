use crate::blockers::BlockerMatrix;
use crate::cards::COMBO_COUNT;
use crate::equity_cache::EquityCache;
use crate::range::ComboRange;
use crate::range_equity::{sampled_equity_vs_range, RangeEquityEstimate};
use crate::terminal::{Seat, TerminalId, TerminalPot};
use crate::terminal_ev::expected_two_active_payoff;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BbHuLeaf {
    AfterBtnFoldSbJam,
    AfterBtnJamSbFold,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BestLeafAction {
    Fold,
    Call,
    Tie,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LeafActionValues {
    pub fold_ev_bb: f64,
    pub call_ev_bb: f64,
    pub call_equity: RangeEquityEstimate,
}

impl LeafActionValues {
    pub fn action_values(self) -> [f64;2] {
        [self.fold_ev_bb,self.call_ev_bb]
    }

    pub fn best_action(self,tolerance:f64) -> BestLeafAction {
        let delta=self.call_ev_bb-self.fold_ev_bb;
        if delta>tolerance { BestLeafAction::Call }
        else if delta< -tolerance { BestLeafAction::Fold }
        else { BestLeafAction::Tie }
    }
}

impl BbHuLeaf {
    fn terminals(self) -> (TerminalId, TerminalId, Seat) {
        match self {
            Self::AfterBtnFoldSbJam => (
                TerminalId::BtnFoldSbJamBbFold,
                TerminalId::BtnFoldSbJamBbCall,
                Seat::Sb,
            ),
            Self::AfterBtnJamSbFold => (
                TerminalId::BtnJamSbFoldBbFold,
                TerminalId::BtnJamSbFoldBbCall,
                Seat::Btn,
            ),
        }
    }
}

pub fn sampled_bb_leaf_action_values(
    leaf: BbHuLeaf,
    stack_bb: f64,
    bb_combo_index: usize,
    jammer_range: &ComboRange,
    blockers: &BlockerMatrix,
    cache: &mut EquityCache,
    samples_per_matchup: u64,
    seed: u64,
) -> Result<LeafActionValues,String> {
    if bb_combo_index>=COMBO_COUNT { return Err("BB combo index out of range".into()); }
    let (fold_id,call_id,jammer)=leaf.terminals();

    let fold_pot=TerminalPot::for_terminal(fold_id,stack_bb);
    let fold_payoff=fold_pot.settle(&[jammer]);
    let fold_ev=fold_payoff[Seat::Bb as usize];

    let equity=sampled_equity_vs_range(
        bb_combo_index,
        jammer_range,
        blockers,
        cache,
        samples_per_matchup,
        seed,
    )?;
    let call_pot=TerminalPot::for_terminal(call_id,stack_bb);
    let call_payoff=expected_two_active_payoff(&call_pot,Seat::Bb,jammer,equity.hero_equity);

    Ok(LeafActionValues{
        fold_ev_bb:fold_ev,
        call_ev_bb:call_payoff[Seat::Bb as usize],
        call_equity:equity,
    })
}

pub fn sampled_bb_leaf_action_vector(
    leaf:BbHuLeaf,
    stack_bb:f64,
    jammer_range:&ComboRange,
    blockers:&BlockerMatrix,
    cache:&mut EquityCache,
    samples_per_matchup:u64,
    seed:u64,
)->Result<Vec<LeafActionValues>,String>{
    let mut out=Vec::with_capacity(COMBO_COUNT);
    for combo_index in 0..COMBO_COUNT {
        out.push(sampled_bb_leaf_action_values(
            leaf,stack_bb,combo_index,jammer_range,blockers,cache,samples_per_matchup,seed
        )?);
    }
    Ok(out)
}

#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn bb_fold_is_minus_one_in_both_supported_leaves(){
        let blockers=BlockerMatrix::build();
        let range=ComboRange::uniform();
        let mut cache=EquityCache::new();
        for leaf in [BbHuLeaf::AfterBtnFoldSbJam,BbHuLeaf::AfterBtnJamSbFold] {
            let v=sampled_bb_leaf_action_values(leaf,8.0,0,&range,&blockers,&mut cache,2,3).unwrap();
            assert_eq!(v.fold_ev_bb,-1.0);
            assert!(v.call_ev_bb.is_finite());
            assert_eq!(v.action_values(),[-1.0,v.call_ev_bb]);
        }
    }

    #[test]
    fn call_ev_matches_equity_times_pot_minus_contribution(){
        let blockers=BlockerMatrix::build();
        let range=ComboRange::uniform();
        let mut cache=EquityCache::new();
        let v=sampled_bb_leaf_action_values(
            BbHuLeaf::AfterBtnFoldSbJam,8.0,0,&range,&blockers,&mut cache,2,9
        ).unwrap();
        assert!((v.call_ev_bb-(16.0*v.call_equity.hero_equity-8.0)).abs()<1e-12);
    }

    #[test]
    fn best_action_respects_tolerance(){
        let eq=RangeEquityEstimate{hero_equity:0.5,compatible_weight:1.0,compatible_combos:1,samples_per_matchup:1,seed:1};
        let fold=LeafActionValues{fold_ev_bb:-1.0,call_ev_bb:-1.2,call_equity:eq};
        let call=LeafActionValues{fold_ev_bb:-1.0,call_ev_bb:-0.8,call_equity:eq};
        let tie=LeafActionValues{fold_ev_bb:-1.0,call_ev_bb:-0.9995,call_equity:eq};
        assert_eq!(fold.best_action(0.001),BestLeafAction::Fold);
        assert_eq!(call.best_action(0.001),BestLeafAction::Call);
        assert_eq!(tie.best_action(0.001),BestLeafAction::Tie);
    }
}

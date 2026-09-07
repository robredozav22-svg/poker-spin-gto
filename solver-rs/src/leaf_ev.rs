use crate::blockers::BlockerMatrix;
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LeafActionValues {
    pub fold_ev_bb: f64,
    pub call_ev_bb: f64,
    pub call_equity: RangeEquityEstimate,
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
        // HU call pot is 16bb, BB contributes 8bb.
        assert!((v.call_ev_bb-(16.0*v.call_equity.hero_equity-8.0)).abs()<1e-12);
    }
}

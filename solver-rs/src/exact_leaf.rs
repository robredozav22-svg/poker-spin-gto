use crate::blockers::BlockerMatrix;
use crate::cards::COMBO_COUNT;
use crate::exact_equity::ExactEquityCache;
use crate::exact_range_equity::{exact_equity_vs_range,ExactRangeEquity};
use crate::leaf_ev::{BbHuLeaf,BestLeafAction};
use crate::range::ComboRange;
use crate::terminal::{Seat,TerminalId,TerminalPot};
use crate::terminal_ev::expected_two_active_payoff;

#[derive(Debug,Clone,Copy,PartialEq)]
pub struct ExactHuLeafActionValues{
    pub fold_ev_bb:f64,
    pub call_ev_bb:f64,
    pub call_equity:ExactRangeEquity,
}

impl ExactHuLeafActionValues{
    pub fn action_values(self)->[f64;2]{[self.fold_ev_bb,self.call_ev_bb]}
    pub fn best_action(self,tolerance:f64)->BestLeafAction{
        let delta=self.call_ev_bb-self.fold_ev_bb;
        if delta>tolerance{BestLeafAction::Call}
        else if delta< -tolerance{BestLeafAction::Fold}
        else{BestLeafAction::Tie}
    }
}

fn terminals(leaf:BbHuLeaf)->(TerminalId,TerminalId,Seat){
    match leaf{
        BbHuLeaf::AfterBtnFoldSbJam=>(
            TerminalId::BtnFoldSbJamBbFold,
            TerminalId::BtnFoldSbJamBbCall,
            Seat::Sb,
        ),
        BbHuLeaf::AfterBtnJamSbFold=>(
            TerminalId::BtnJamSbFoldBbFold,
            TerminalId::BtnJamSbFoldBbCall,
            Seat::Btn,
        ),
    }
}

/// Canonical exact HU BB decision for supported all-in leaves.
/// Opponent range is conditioned on the exact BB cards and every legal board
/// is enumerated through ExactEquityCache.
pub fn exact_bb_leaf_action_values(
    leaf:BbHuLeaf,
    stack_bb:f64,
    bb_combo_index:usize,
    jammer_range:&ComboRange,
    blockers:&BlockerMatrix,
    cache:&mut ExactEquityCache,
)->Result<ExactHuLeafActionValues,String>{
    if bb_combo_index>=COMBO_COUNT{return Err("BB combo index out of range".into());}
    let (fold_id,call_id,jammer)=terminals(leaf);
    let fold_pot=TerminalPot::for_terminal(fold_id,stack_bb);
    let fold_ev=fold_pot.settle(&[jammer])[Seat::Bb as usize];
    let equity=exact_equity_vs_range(bb_combo_index,jammer_range,blockers,cache)?;
    let call_pot=TerminalPot::for_terminal(call_id,stack_bb);
    let payoff=expected_two_active_payoff(&call_pot,Seat::Bb,jammer,equity.hero_equity);
    Ok(ExactHuLeafActionValues{
        fold_ev_bb:fold_ev,
        call_ev_bb:payoff[Seat::Bb as usize],
        call_equity:equity,
    })
}

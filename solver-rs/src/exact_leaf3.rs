use crate::blockers::BlockerMatrix;
use crate::cards::COMBO_COUNT;
use crate::exact_equity3::ExactThreeWayEquityCache;
use crate::exact_range_equity3::{exact_threeway_equity_vs_ranges,ExactThreeWayRangeEquity};
use crate::payoff_lookup::ThreeWayPayoffLookup;
use crate::persisted_range_equity::persisted_threeway_equity_vs_ranges;
use crate::range::ComboRange;
use crate::terminal::{Seat,TerminalId,TerminalPot};
use crate::terminal_ev::expected_three_active_payoff;

#[derive(Debug,Clone,Copy,PartialEq)]
pub struct ExactThreeWayLeafActionValues{
    pub fold_ev_bb:f64,
    pub call_ev_bb:f64,
    /// Equity order is [BB, BTN, SB] from the exact range integrator.
    pub call_equity:ExactThreeWayRangeEquity,
}

impl ExactThreeWayLeafActionValues{
    pub fn action_values(self)->[f64;2]{[self.fold_ev_bb,self.call_ev_bb]}
}

fn settle_leaf(
    stack_bb:f64,
    eq:ExactThreeWayRangeEquity,
)->Result<ExactThreeWayLeafActionValues,String>{
    let fold_pot=TerminalPot::for_terminal(TerminalId::BtnJamSbCallBbFold,stack_bb);
    let fold_ev=fold_pot.settle(&[Seat::Btn])[Seat::Bb as usize];
    let call_pot=TerminalPot::for_terminal(TerminalId::BtnJamSbCallBbCall,stack_bb);
    let seat_order=[eq.equities[1],eq.equities[2],eq.equities[0]];
    let payoff=expected_three_active_payoff(&call_pot,seat_order)?;
    Ok(ExactThreeWayLeafActionValues{
        fold_ev_bb:fold_ev,
        call_ev_bb:payoff[Seat::Bb as usize],
        call_equity:eq,
    })
}

/// Build/research exact BB decision after BTN jam and SB call. Required board
/// enumerations are computed through ExactThreeWayEquityCache.
pub fn exact_bb_after_btn_jam_sb_call_values(
    stack_bb:f64,
    bb_combo_index:usize,
    btn_jam_range:&ComboRange,
    sb_call_range:&ComboRange,
    blockers:&BlockerMatrix,
    cache:&mut ExactThreeWayEquityCache,
)->Result<ExactThreeWayLeafActionValues,String>{
    if bb_combo_index>=COMBO_COUNT{return Err("BB combo index out of range".into());}
    let eq=exact_threeway_equity_vs_ranges(
        bb_combo_index,btn_jam_range,sb_call_range,blockers,cache
    )?;
    settle_leaf(stack_bb,eq)
}

/// Runtime persisted exact BB decision after BTN jam and SB call. Every jointly
/// compatible positive-mass BTN/SB matchup must already be present in `lookup`.
/// Missing exact payoff data fails closed; no direct or sampled fallback occurs.
pub fn persisted_bb_after_btn_jam_sb_call_values(
    stack_bb:f64,
    bb_combo_index:usize,
    btn_jam_range:&ComboRange,
    sb_call_range:&ComboRange,
    blockers:&BlockerMatrix,
    lookup:&ThreeWayPayoffLookup,
)->Result<ExactThreeWayLeafActionValues,String>{
    if bb_combo_index>=COMBO_COUNT{return Err("BB combo index out of range".into());}
    let eq=persisted_threeway_equity_vs_ranges(
        bb_combo_index,btn_jam_range,sb_call_range,blockers,lookup
    )?;
    settle_leaf(stack_bb,eq)
}

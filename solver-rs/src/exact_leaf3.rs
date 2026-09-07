use crate::blockers::BlockerMatrix;
use crate::cards::COMBO_COUNT;
use crate::exact_equity3::ExactThreeWayEquityCache;
use crate::exact_range_equity3::{exact_threeway_equity_vs_ranges,ExactThreeWayRangeEquity};
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

/// Exact BB decision after BTN jam and SB call.
/// The call branch integrates the jointly compatible BTN and SB ranges over
/// every legal board. No Monte Carlo and no pairwise-equity substitution.
pub fn exact_bb_after_btn_jam_sb_call_values(
    stack_bb:f64,
    bb_combo_index:usize,
    btn_jam_range:&ComboRange,
    sb_call_range:&ComboRange,
    blockers:&BlockerMatrix,
    cache:&mut ExactThreeWayEquityCache,
)->Result<ExactThreeWayLeafActionValues,String>{
    if bb_combo_index>=COMBO_COUNT{return Err("BB combo index out of range".into());}
    let fold_pot=TerminalPot::for_terminal(TerminalId::BtnJamSbCallBbFold,stack_bb);
    let fold_ev=fold_pot.settle(&[Seat::Btn])[Seat::Bb as usize];

    let eq=exact_threeway_equity_vs_ranges(
        bb_combo_index,btn_jam_range,sb_call_range,blockers,cache
    )?;
    let call_pot=TerminalPot::for_terminal(TerminalId::BtnJamSbCallBbCall,stack_bb);
    let seat_order=[eq.equities[1],eq.equities[2],eq.equities[0]];
    let payoff=expected_three_active_payoff(&call_pot,seat_order)?;
    Ok(ExactThreeWayLeafActionValues{
        fold_ev_bb:fold_ev,
        call_ev_bb:payoff[Seat::Bb as usize],
        call_equity:eq,
    })
}

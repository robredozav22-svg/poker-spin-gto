use crate::blockers::BlockerMatrix;
use crate::cards::COMBO_COUNT;
use crate::equity_cache::EquityCache;
use crate::range::ComboRange;
use crate::range_equity::sampled_equity_vs_range;
use crate::terminal::{Seat,TerminalId,TerminalPot};
use crate::terminal_ev::expected_two_active_payoff;

#[derive(Debug,Clone,Copy,PartialEq)]
pub struct BbVsBtnJamValues{
    pub fold_ev:f64,
    pub call_ev:f64,
    pub hero_equity:f64,
    pub compatible_btn_combos:usize,
}

pub fn bb_vs_btn_jam_values(
    bb_combo_index:usize,
    btn_jam_range:&ComboRange,
    blockers:&BlockerMatrix,
    cache:&mut EquityCache,
    stack_bb:f64,
    samples_per_matchup:u64,
    seed:u64,
)->Result<BbVsBtnJamValues,String>{
    if bb_combo_index>=COMBO_COUNT{return Err("BB combo index out of range".into());}
    let fold_terminal=TerminalPot::for_terminal(TerminalId::BtnJamSbFoldBbFold,stack_bb);
    let fold_ev=fold_terminal.settle(&[Seat::Btn])[Seat::Bb as usize];

    let eq=sampled_equity_vs_range(
        bb_combo_index,
        btn_jam_range,
        blockers,
        cache,
        samples_per_matchup,
        seed,
    )?;
    let call_terminal=TerminalPot::for_terminal(TerminalId::BtnJamSbFoldBbCall,stack_bb);
    let call_payoff=expected_two_active_payoff(&call_terminal,Seat::Bb,Seat::Btn,eq.hero_equity);

    Ok(BbVsBtnJamValues{
        fold_ev,
        call_ev:call_payoff[Seat::Bb as usize],
        hero_equity:eq.hero_equity,
        compatible_btn_combos:eq.compatible_combos,
    })
}

pub fn bb_vs_btn_jam_vector(
    btn_jam_range:&ComboRange,
    blockers:&BlockerMatrix,
    cache:&mut EquityCache,
    stack_bb:f64,
    samples_per_matchup:u64,
    seed:u64,
)->Result<Vec<BbVsBtnJamValues>,String>{
    let mut out=Vec::with_capacity(COMBO_COUNT);
    for bb_combo_index in 0..COMBO_COUNT{
        out.push(bb_vs_btn_jam_values(
            bb_combo_index,btn_jam_range,blockers,cache,stack_bb,samples_per_matchup,seed
        )?);
    }
    Ok(out)
}

#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn fold_ev_is_exact_big_blind_loss(){
        let blockers=BlockerMatrix::build();
        let range=ComboRange::uniform();
        let mut cache=EquityCache::new();
        let v=bb_vs_btn_jam_values(0,&range,&blockers,&mut cache,8.0,1,17).unwrap();
        assert_eq!(v.fold_ev,-1.0);
        assert_eq!(v.compatible_btn_combos,1225);
        assert!((0.0..=1.0).contains(&v.hero_equity));
        assert!(v.call_ev.is_finite());
    }

    #[test]
    fn call_ev_matches_equity_pot_formula(){
        let blockers=BlockerMatrix::build();
        let range=ComboRange::uniform();
        let mut cache=EquityCache::new();
        let v=bb_vs_btn_jam_values(25,&range,&blockers,&mut cache,8.0,1,31).unwrap();
        // BTN 8 + dead SB .5 + BB 8 = 16.5 total; BB invested 8.
        let expected=v.hero_equity*16.5-8.0;
        assert!((v.call_ev-expected).abs()<1e-12);
    }
}

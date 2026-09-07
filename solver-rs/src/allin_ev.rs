use crate::cards::Combo;
use crate::equity_cache::EquityCache;
use crate::terminal::{Seat, TerminalId, TerminalPot};
use crate::terminal_ev::expected_hu_payoff;

pub fn sampled_hu_allin_ev(
    cache: &mut EquityCache,
    terminal: TerminalId,
    stack_bb: f64,
    hero_seat: Seat,
    villain_seat: Seat,
    hero_combo: Combo,
    villain_combo: Combo,
    samples: u64,
    seed: u64,
) -> Result<[f64;3],String> {
    let pot=TerminalPot::for_terminal(terminal,stack_bb);
    if pot.active.iter().filter(|v|**v).count()!=2 {
        return Err("sampled_hu_allin_ev requires a two-active-player terminal".into());
    }
    let equity=cache.get_or_compute(hero_combo,villain_combo,samples,seed)?;
    expected_hu_payoff(&pot,hero_seat,villain_seat,equity)
}

#[cfg(test)]
mod tests{
    use super::*;
    use crate::cards::Card;

    fn c(r:u8,s:u8)->Card{r*4+s}

    #[test]
    fn cached_allin_ev_is_reproducible_and_zero_sum(){
        let mut cache=EquityCache::new();
        let h=[c(12,0),c(12,1)];
        let v=[c(11,2),c(11,3)];
        let a=sampled_hu_allin_ev(
            &mut cache,
            TerminalId::BtnJamSbFoldBbCall,
            8.0,
            Seat::Btn,
            Seat::Bb,
            h,v,1000,99
        ).unwrap();
        let b=sampled_hu_allin_ev(
            &mut cache,
            TerminalId::BtnJamSbFoldBbCall,
            8.0,
            Seat::Btn,
            Seat::Bb,
            h,v,1000,99
        ).unwrap();
        assert_eq!(a,b);
        assert!(a.iter().sum::<f64>().abs()<1e-12);
        assert_eq!(cache.misses(),1);
        assert_eq!(cache.hits(),1);
    }

    #[test]
    fn rejects_three_way_terminal_instead_of_pairwise_substitution(){
        let mut cache=EquityCache::new();
        let h=[c(12,0),c(12,1)];
        let v=[c(11,2),c(11,3)];
        let err=sampled_hu_allin_ev(
            &mut cache,
            TerminalId::BtnJamSbCallBbCall,
            8.0,
            Seat::Btn,
            Seat::Sb,
            h,v,100,1
        ).unwrap_err();
        assert!(err.contains("two-active-player"));
        assert_eq!(cache.len(),0);
    }
}

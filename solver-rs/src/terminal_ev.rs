use crate::terminal::{Seat, TerminalPot};

pub fn expected_two_active_payoff(
    pot: &TerminalPot,
    hero: Seat,
    villain: Seat,
    hero_equity: f64,
) -> [f64;3] {
    assert!((0.0..=1.0).contains(&hero_equity));
    assert!(pot.active[hero as usize]);
    assert!(pot.active[villain as usize]);
    assert_ne!(hero as usize,villain as usize);
    assert_eq!(pot.active.iter().filter(|v|**v).count(),2);

    let total=pot.pot();
    let mut payoff=[0.0;3];
    for i in 0..3 { payoff[i]=-pot.contributions[i]; }
    payoff[hero as usize]+=hero_equity*total;
    payoff[villain as usize]+=(1.0-hero_equity)*total;
    payoff
}

#[cfg(test)]
mod tests{
    use super::*;
    use crate::terminal::TerminalId;

    #[test]
    fn expected_payoff_is_zero_sum_with_dead_money(){
        let pot=TerminalPot::for_terminal(TerminalId::BtnJamSbCallBbFold,8.0);
        let ev=expected_two_active_payoff(&pot,Seat::Btn,Seat::Sb,0.60);
        assert!(ev.iter().sum::<f64>().abs()<1e-12);
        // pot=17; BTN contribution=8 => .6*17-8 = +2.2
        assert!((ev[0]-2.2).abs()<1e-12);
        // SB gets 40% of 17 minus 8 = -1.2; BB loses blind.
        assert!((ev[1]+1.2).abs()<1e-12);
        assert_eq!(ev[2],-1.0);
    }

    #[test]
    fn fifty_percent_equity_splits_dead_blind(){
        let pot=TerminalPot::for_terminal(TerminalId::BtnJamSbCallBbFold,8.0);
        let ev=expected_two_active_payoff(&pot,Seat::Btn,Seat::Sb,0.5);
        assert_eq!(ev,[0.5,0.5,-1.0]);
    }
}

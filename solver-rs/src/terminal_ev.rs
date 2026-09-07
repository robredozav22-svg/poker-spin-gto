use crate::equity::EquityEstimate;
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

/// Strict bridge from a measured/sampled HU equity object to chip-EV.
/// This intentionally rejects three-way terminals rather than substituting
/// pairwise equities for a multiway showdown.
pub fn expected_hu_payoff(
    pot: &TerminalPot,
    hero: Seat,
    villain: Seat,
    equity: EquityEstimate,
) -> Result<[f64;3],String> {
    if hero == villain { return Err("hero and villain must be different seats".into()); }
    if !pot.active[hero as usize] || !pot.active[villain as usize] {
        return Err("hero and villain must both be active".into());
    }
    let active_count=pot.active.iter().filter(|v|**v).count();
    if active_count!=2 {
        return Err(format!("heads-up payoff requires exactly 2 active seats, got {active_count}"));
    }
    if !equity.hero.is_finite() || !equity.villain.is_finite() {
        return Err("equity must be finite".into());
    }
    if equity.hero<0.0 || equity.hero>1.0 || equity.villain<0.0 || equity.villain>1.0 {
        return Err("equity outside [0,1]".into());
    }
    if (equity.hero+equity.villain-1.0).abs()>1e-9 {
        return Err("heads-up equities must sum to 1".into());
    }
    Ok(expected_two_active_payoff(pot,hero,villain,equity.hero))
}

#[cfg(test)]
mod tests{
    use super::*;
    use crate::terminal::TerminalId;

    fn eq(hero:f64)->EquityEstimate{
        EquityEstimate{hero,villain:1.0-hero,ties:0,samples:1000,seed:1}
    }

    #[test]
    fn expected_payoff_is_zero_sum_with_dead_money(){
        let pot=TerminalPot::for_terminal(TerminalId::BtnJamSbCallBbFold,8.0);
        let ev=expected_two_active_payoff(&pot,Seat::Btn,Seat::Sb,0.60);
        assert!(ev.iter().sum::<f64>().abs()<1e-12);
        assert!((ev[0]-2.2).abs()<1e-12);
        assert!((ev[1]+1.2).abs()<1e-12);
        assert_eq!(ev[2],-1.0);
    }

    #[test]
    fn fifty_percent_equity_splits_dead_blind(){
        let pot=TerminalPot::for_terminal(TerminalId::BtnJamSbCallBbFold,8.0);
        let ev=expected_hu_payoff(&pot,Seat::Btn,Seat::Sb,eq(0.5)).unwrap();
        assert_eq!(ev,[0.5,0.5,-1.0]);
    }

    #[test]
    fn certain_winner_matches_exact_settlement(){
        let pot=TerminalPot::for_terminal(TerminalId::BtnJamSbCallBbFold,8.0);
        let ev=expected_hu_payoff(&pot,Seat::Btn,Seat::Sb,eq(1.0)).unwrap();
        assert_eq!(ev,pot.settle(&[Seat::Btn]));
    }

    #[test]
    fn rejects_three_way_terminal(){
        let pot=TerminalPot::for_terminal(TerminalId::BtnJamSbCallBbCall,8.0);
        assert!(expected_hu_payoff(&pot,Seat::Btn,Seat::Sb,eq(0.5)).is_err());
    }
}

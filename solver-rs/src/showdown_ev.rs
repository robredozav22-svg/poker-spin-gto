use crate::equity::EquityEstimate;
use crate::terminal::{Seat, TerminalPot};

/// Convert a heads-up equity estimate into expected chip-EV at a terminal pot.
/// Exactly two active seats must correspond to hero and villain. Any folded
/// player's dead contribution remains a deterministic loss in the payoff.
pub fn expected_hu_payoff(
    pot: &TerminalPot,
    hero: Seat,
    villain: Seat,
    equity: EquityEstimate,
) -> Result<[f64; 3], String> {
    let hi = hero as usize;
    let vi = villain as usize;
    if hero == villain {
        return Err("hero and villain must be different seats".into());
    }
    if !pot.active[hi] || !pot.active[vi] {
        return Err("hero and villain must both be active".into());
    }
    let active_count = pot.active.iter().filter(|x| **x).count();
    if active_count != 2 {
        return Err(format!("heads-up payoff requires exactly 2 active seats, got {active_count}"));
    }
    if !equity.hero.is_finite() || !equity.villain.is_finite() {
        return Err("equity must be finite".into());
    }
    if (equity.hero + equity.villain - 1.0).abs() > 1e-9 {
        return Err("heads-up equities must sum to 1".into());
    }
    if equity.hero < 0.0 || equity.hero > 1.0 || equity.villain < 0.0 || equity.villain > 1.0 {
        return Err("equity outside [0,1]".into());
    }

    let total_pot = pot.pot();
    let mut payoff = [0.0; 3];
    for i in 0..3 {
        payoff[i] = -pot.contributions[i];
    }
    payoff[hi] += equity.hero * total_pot;
    payoff[vi] += equity.villain * total_pot;
    Ok(payoff)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::terminal::{TerminalId, TerminalPot};

    fn eq(hero: f64) -> EquityEstimate {
        EquityEstimate { hero, villain: 1.0 - hero, ties: 0, samples: 1000, seed: 1 }
    }

    #[test]
    fn fifty_fifty_btn_vs_bb_keeps_dead_sb_blind_in_ev() {
        let pot = TerminalPot::for_terminal(TerminalId::BtnJamSbFoldBbCall, 8.0);
        let p = expected_hu_payoff(&pot, Seat::Btn, Seat::Bb, eq(0.5)).unwrap();
        assert!((p[0] - 0.25).abs() < 1e-12);
        assert!((p[1] + 0.5).abs() < 1e-12);
        assert!((p[2] - 0.25).abs() < 1e-12);
        assert!(p.iter().sum::<f64>().abs() < 1e-12);
    }

    #[test]
    fn certain_winner_matches_terminal_settlement() {
        let pot = TerminalPot::for_terminal(TerminalId::BtnJamSbCallBbFold, 8.0);
        let expected = expected_hu_payoff(&pot, Seat::Btn, Seat::Sb, eq(1.0)).unwrap();
        let exact = pot.settle(&[Seat::Btn]);
        assert_eq!(expected, exact);
    }

    #[test]
    fn rejects_three_way_terminal() {
        let pot = TerminalPot::for_terminal(TerminalId::BtnJamSbCallBbCall, 8.0);
        assert!(expected_hu_payoff(&pot, Seat::Btn, Seat::Sb, eq(0.5)).is_err());
    }
}

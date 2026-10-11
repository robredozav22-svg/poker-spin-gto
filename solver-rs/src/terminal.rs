#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Seat {
    Btn = 0,
    Sb = 1,
    Bb = 2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerminalId {
    BtnFoldSbFold,
    BtnFoldSbJamBbFold,
    BtnFoldSbJamBbCall,
    BtnJamSbFoldBbFold,
    BtnJamSbFoldBbCall,
    BtnJamSbCallBbFold,
    BtnJamSbCallBbCall,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TerminalPot {
    pub contributions: [f64; 3],
    pub active: [bool; 3],
}

impl TerminalPot {
    pub fn for_terminal(id: TerminalId, stack_bb: f64) -> Self {
        assert!(stack_bb >= 1.0);
        match id {
            TerminalId::BtnFoldSbFold => Self {
                contributions: [0.0, 0.5, 1.0],
                active: [false, false, true],
            },
            // SB's unmatched jam is returned; only 1bb needs to remain committed
            // to win BB's blind after BTN already folded.
            TerminalId::BtnFoldSbJamBbFold => Self {
                contributions: [0.0, 1.0, 1.0],
                active: [false, true, false],
            },
            TerminalId::BtnFoldSbJamBbCall => Self {
                contributions: [0.0, stack_bb, stack_bb],
                active: [false, true, true],
            },
            // BTN's unmatched jam is returned down to the largest live blind (1bb).
            TerminalId::BtnJamSbFoldBbFold => Self {
                contributions: [1.0, 0.5, 1.0],
                active: [true, false, false],
            },
            TerminalId::BtnJamSbFoldBbCall => Self {
                contributions: [stack_bb, 0.5, stack_bb],
                active: [true, false, true],
            },
            TerminalId::BtnJamSbCallBbFold => Self {
                contributions: [stack_bb, stack_bb, 1.0],
                active: [true, true, false],
            },
            TerminalId::BtnJamSbCallBbCall => Self {
                contributions: [stack_bb, stack_bb, stack_bb],
                active: [true, true, true],
            },
        }
    }

    pub fn pot(&self) -> f64 {
        self.contributions.iter().sum()
    }

    pub fn settle(&self, winners: &[Seat]) -> [f64; 3] {
        assert!(!winners.is_empty());
        for winner in winners {
            assert!(self.active[*winner as usize], "winner must be active");
        }
        let mut unique = [false; 3];
        for winner in winners {
            unique[*winner as usize] = true;
        }
        let winner_count = unique.iter().filter(|v| **v).count();
        assert!(winner_count > 0);
        let share = self.pot() / winner_count as f64;
        let mut payoff = [0.0; 3];
        for i in 0..3 {
            payoff[i] = -self.contributions[i];
            if unique[i] {
                payoff[i] += share;
            }
        }
        payoff
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn zero_sum(payoff: [f64; 3]) {
        assert!(payoff.iter().sum::<f64>().abs() < 1e-12, "{payoff:?}");
    }

    #[test]
    fn blind_walk_is_correct() {
        let p = TerminalPot::for_terminal(TerminalId::BtnFoldSbFold, 8.0);
        let payoff = p.settle(&[Seat::Bb]);
        assert_eq!(payoff, [0.0, -0.5, 0.5]);
        zero_sum(payoff);
    }

    #[test]
    fn uncalled_sb_jam_returns_excess() {
        let p = TerminalPot::for_terminal(TerminalId::BtnFoldSbJamBbFold, 8.0);
        let payoff = p.settle(&[Seat::Sb]);
        assert_eq!(payoff, [0.0, 1.0, -1.0]);
        zero_sum(payoff);
    }

    #[test]
    fn btn_open_jam_wins_blinds_when_both_fold() {
        let p = TerminalPot::for_terminal(TerminalId::BtnJamSbFoldBbFold, 8.0);
        let payoff = p.settle(&[Seat::Btn]);
        assert_eq!(payoff, [1.5, -0.5, -1.0]);
        zero_sum(payoff);
    }

    #[test]
    fn heads_up_allin_with_dead_bb_blind() {
        let p = TerminalPot::for_terminal(TerminalId::BtnJamSbCallBbFold, 8.0);
        let btn_win = p.settle(&[Seat::Btn]);
        assert_eq!(btn_win, [9.0, -8.0, -1.0]);
        zero_sum(btn_win);
        let tie = p.settle(&[Seat::Btn, Seat::Sb]);
        assert_eq!(tie, [0.5, 0.5, -1.0]);
        zero_sum(tie);
    }

    #[test]
    fn three_way_allin_settlement_is_zero_sum() {
        let p = TerminalPot::for_terminal(TerminalId::BtnJamSbCallBbCall, 8.0);
        let btn_win = p.settle(&[Seat::Btn]);
        assert_eq!(btn_win, [16.0, -8.0, -8.0]);
        zero_sum(btn_win);
        let three_way_tie = p.settle(&[Seat::Btn, Seat::Sb, Seat::Bb]);
        assert_eq!(three_way_tie, [0.0, 0.0, 0.0]);
        zero_sum(three_way_tie);
    }
}

pub mod blockers;
pub mod cards;
pub mod equity;
pub mod equity_cache;
pub mod evaluator;
pub mod joint;
pub mod leaf_ev;
pub mod range;
pub mod range_equity;
pub mod regret;
pub mod terminal;
pub mod terminal_ev;
pub mod tree;

pub const HAND_CLASSES: usize = 169;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SolverStatus {
    ResearchOnly,
    StableApprox,
    Promoted,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrategySnapshot {
    pub infosets: usize,
    pub actions: usize,
    pub probabilities: Vec<f64>,
}

impl StrategySnapshot {
    pub fn row(&self, infoset: usize) -> &[f64] {
        let start = infoset * self.actions;
        &self.probabilities[start..start + self.actions]
    }
}

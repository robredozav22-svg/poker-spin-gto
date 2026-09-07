pub mod blockers;
pub mod cards;
pub mod class_aggregation;
pub mod continuation_registry;
pub mod equity;
pub mod equity3;
pub mod equity3_cache;
pub mod equity_cache;
pub mod evaluator;
pub mod exact_equity;
pub mod exact_equity3;
pub mod exact_leaf;
pub mod exact_leaf3;
pub mod exact_range_equity;
pub mod exact_range_equity3;
pub mod joint;
pub mod leaf_cfr;
pub mod leaf_ev;
pub mod matrix_game;
pub mod payoff_build;
pub mod payoff_lookup;
pub mod payoff_manifest;
pub mod payoff_table;
pub mod persisted_range_equity;
pub mod preflop_tree;
pub mod range;
pub mod range_equity;
pub mod range_equity3;
pub mod reference_tree_15bb;
pub mod regret;
pub mod restricted_eval;
pub mod restricted_exact;
pub mod restricted_subgame;
pub mod sb_ev;
pub mod solver_evidence;
pub mod strategy_compare;
pub mod strategy_metrics;
pub mod strategy_range;
pub mod strategy_record;
pub mod terminal;
pub mod terminal_ev;
pub mod tree;
pub mod tree_catalog;
pub mod tree_profile;

pub const HAND_CLASSES:usize=169;

#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum SolverStatus{ResearchOnly,StableApprox,Promoted}

#[derive(Debug,Clone,PartialEq)]
pub struct StrategySnapshot{pub infosets:usize,pub actions:usize,pub probabilities:Vec<f64>}

impl StrategySnapshot{
    pub fn row(&self,infoset:usize)->&[f64]{let start=infoset*self.actions;&self.probabilities[start..start+self.actions]}
}

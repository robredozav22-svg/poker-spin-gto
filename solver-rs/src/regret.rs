use crate::StrategySnapshot;

#[derive(Debug, Clone)]
pub struct RegretTable {
    infosets: usize,
    actions: usize,
    regrets: Vec<f64>,
    strategy_sum: Vec<f64>,
}

impl RegretTable {
    pub fn new(infosets: usize, actions: usize) -> Self {
        assert!(infosets > 0);
        assert!(actions > 1);
        Self {
            infosets,
            actions,
            regrets: vec![0.0; infosets * actions],
            strategy_sum: vec![0.0; infosets * actions],
        }
    }

    pub fn current_strategy(&self, infoset: usize) -> Vec<f64> {
        let start = infoset * self.actions;
        let row = &self.regrets[start..start + self.actions];
        let positive_sum: f64 = row.iter().map(|r| r.max(0.0)).sum();
        if positive_sum <= f64::EPSILON {
            return vec![1.0 / self.actions as f64; self.actions];
        }
        row.iter().map(|r| r.max(0.0) / positive_sum).collect()
    }

    pub fn add_regret_row(&mut self, infoset: usize, delta: &[f64]) {
        assert_eq!(delta.len(), self.actions);
        let start = infoset * self.actions;
        for (i, value) in delta.iter().enumerate() {
            self.regrets[start + i] += value;
        }
    }

    pub fn accumulate_strategy_row(&mut self, infoset: usize, strategy: &[f64], weight: f64) {
        assert_eq!(strategy.len(), self.actions);
        let start = infoset * self.actions;
        for (i, value) in strategy.iter().enumerate() {
            self.strategy_sum[start + i] += value * weight;
        }
    }

    /// Apply one full action-value CFR update for an infoset.
    ///
    /// `action_values[a]` is the counterfactual value of taking action `a`.
    /// The current mixed strategy is computed from regrets, the node value is
    /// the strategy-weighted average, and each regret receives
    /// `opponent_reach * (action_value - node_value)`. Strategy averaging uses
    /// `own_reach`, kept separate from regret weighting.
    pub fn update_from_action_values(
        &mut self,
        infoset: usize,
        action_values: &[f64],
        opponent_reach: f64,
        own_reach: f64,
    ) -> f64 {
        assert_eq!(action_values.len(), self.actions);
        let strategy = self.current_strategy(infoset);
        let node_value: f64 = strategy
            .iter()
            .zip(action_values.iter())
            .map(|(p, v)| p * v)
            .sum();
        let delta: Vec<f64> = action_values
            .iter()
            .map(|v| opponent_reach * (v - node_value))
            .collect();
        self.add_regret_row(infoset, &delta);
        self.accumulate_strategy_row(infoset, &strategy, own_reach);
        node_value
    }

    pub fn dcfr_discount(&mut self, iteration: u64, alpha: f64, beta: f64, gamma: f64) {
        assert!(iteration > 0);
        let t = iteration as f64;
        let pos = t.powf(alpha) / (t.powf(alpha) + 1.0);
        let neg = t.powf(beta) / (t.powf(beta) + 1.0);
        let strat = (t / (t + 1.0)).powf(gamma);

        for r in &mut self.regrets {
            *r *= if *r >= 0.0 { pos } else { neg };
        }
        for s in &mut self.strategy_sum {
            *s *= strat;
        }
    }

    pub fn cfr_plus_floor(&mut self) {
        for r in &mut self.regrets {
            if *r < 0.0 {
                *r = 0.0;
            }
        }
    }

    pub fn average_strategy(&self) -> StrategySnapshot {
        let mut probabilities = vec![0.0; self.infosets * self.actions];
        for infoset in 0..self.infosets {
            let start = infoset * self.actions;
            let row = &self.strategy_sum[start..start + self.actions];
            let sum: f64 = row.iter().sum();
            if sum <= f64::EPSILON {
                let uniform = 1.0 / self.actions as f64;
                probabilities[start..start + self.actions].fill(uniform);
            } else {
                for i in 0..self.actions {
                    probabilities[start + i] = row[i] / sum;
                }
            }
        }
        StrategySnapshot {
            infosets: self.infosets,
            actions: self.actions,
            probabilities,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_regrets_are_uniform() {
        let table = RegretTable::new(2, 3);
        assert_eq!(table.current_strategy(0), vec![1.0 / 3.0; 3]);
    }

    #[test]
    fn positive_regrets_drive_strategy() {
        let mut table = RegretTable::new(1, 2);
        table.add_regret_row(0, &[3.0, 1.0]);
        let s = table.current_strategy(0);
        assert!((s[0] - 0.75).abs() < 1e-12);
        assert!((s[1] - 0.25).abs() < 1e-12);
    }

    #[test]
    fn full_action_value_update_moves_toward_better_action() {
        let mut table = RegretTable::new(1, 2);
        let node_value = table.update_from_action_values(0, &[1.0, 3.0], 1.0, 1.0);
        assert!((node_value - 2.0).abs() < 1e-12);
        let s = table.current_strategy(0);
        assert_eq!(s, vec![0.0, 1.0]);
    }

    #[test]
    fn cfr_plus_removes_negative_regrets() {
        let mut table = RegretTable::new(1, 2);
        table.add_regret_row(0, &[-2.0, 1.0]);
        table.cfr_plus_floor();
        let s = table.current_strategy(0);
        assert!((s[0] - 0.0).abs() < 1e-12);
        assert!((s[1] - 1.0).abs() < 1e-12);
    }

    #[test]
    fn average_strategy_is_normalized() {
        let mut table = RegretTable::new(1, 2);
        table.accumulate_strategy_row(0, &[0.8, 0.2], 2.0);
        table.accumulate_strategy_row(0, &[0.4, 0.6], 1.0);
        let avg = table.average_strategy();
        let row = avg.row(0);
        assert!((row.iter().sum::<f64>() - 1.0).abs() < 1e-12);
        assert!((row[0] - 2.0 / 3.0).abs() < 1e-12);
    }
}

use crate::blockers::BlockerMatrix;
use crate::cards::COMBO_COUNT;

#[derive(Debug, Clone, PartialEq)]
pub struct ComboRange {
    weights: Vec<f64>,
}

impl ComboRange {
    pub fn uniform() -> Self {
        Self { weights: vec![1.0 / COMBO_COUNT as f64; COMBO_COUNT] }
    }

    pub fn from_weights(weights: Vec<f64>) -> Result<Self, String> {
        if weights.len() != COMBO_COUNT {
            return Err(format!("expected {COMBO_COUNT} combo weights, got {}", weights.len()));
        }
        if weights.iter().any(|w| !w.is_finite() || *w < 0.0) {
            return Err("range contains invalid weight".into());
        }
        let sum: f64 = weights.iter().sum();
        if sum <= f64::EPSILON {
            return Err("range weight sum is zero".into());
        }
        Ok(Self { weights: weights.into_iter().map(|w| w / sum).collect() })
    }

    pub fn weights(&self) -> &[f64] {
        &self.weights
    }

    pub fn conditioned_on_blockers(
        &self,
        hero_combo: usize,
        blockers: &BlockerMatrix,
    ) -> Result<Vec<f64>, String> {
        if hero_combo >= COMBO_COUNT {
            return Err("hero combo index out of range".into());
        }
        let mut out = vec![0.0; COMBO_COUNT];
        let mut sum = 0.0;
        for j in 0..COMBO_COUNT {
            if blockers.compatible(hero_combo, j) {
                out[j] = self.weights[j];
                sum += out[j];
            }
        }
        if sum <= f64::EPSILON {
            return Err("no compatible villain combos with positive weight".into());
        }
        for w in &mut out {
            *w /= sum;
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uniform_range_conditions_to_1225_equal_combos() {
        let blockers = BlockerMatrix::build();
        let range = ComboRange::uniform();
        let c = range.conditioned_on_blockers(0, &blockers).unwrap();
        let positive: Vec<f64> = c.iter().copied().filter(|w| *w > 0.0).collect();
        assert_eq!(positive.len(), 1225);
        assert!((c.iter().sum::<f64>() - 1.0).abs() < 1e-12);
        for w in positive.iter().take(20) {
            assert!((*w - 1.0 / 1225.0).abs() < 1e-12);
        }
    }

    #[test]
    fn range_normalizes_input_weights() {
        let mut weights = vec![0.0; COMBO_COUNT];
        weights[100] = 2.0;
        weights[200] = 1.0;
        let range = ComboRange::from_weights(weights).unwrap();
        assert!((range.weights()[100] - 2.0 / 3.0).abs() < 1e-12);
        assert!((range.weights()[200] - 1.0 / 3.0).abs() < 1e-12);
    }
}

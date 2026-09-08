use crate::cards::{all_combos, disjoint, COMBO_COUNT};

const WORD_BITS: usize = 64;
const WORDS_PER_ROW: usize = (COMBO_COUNT + WORD_BITS - 1) / WORD_BITS;

#[derive(Debug, Clone)]
pub struct BlockerMatrix {
    bits: Vec<u64>,
}

impl BlockerMatrix {
    pub fn build() -> Self {
        let combos = all_combos();
        let mut bits = vec![0u64; COMBO_COUNT * WORDS_PER_ROW];
        for i in 0..COMBO_COUNT {
            for j in 0..COMBO_COUNT {
                if disjoint(combos[i], combos[j]) {
                    let idx = i * WORDS_PER_ROW + j / WORD_BITS;
                    bits[idx] |= 1u64 << (j % WORD_BITS);
                }
            }
        }
        Self { bits }
    }

    pub fn compatible(&self, hero_combo: usize, villain_combo: usize) -> bool {
        assert!(hero_combo < COMBO_COUNT && villain_combo < COMBO_COUNT);
        let idx = hero_combo * WORDS_PER_ROW + villain_combo / WORD_BITS;
        (self.bits[idx] & (1u64 << (villain_combo % WORD_BITS))) != 0
    }

    pub fn compatible_count(&self, hero_combo: usize) -> usize {
        assert!(hero_combo < COMBO_COUNT);
        let start = hero_combo * WORDS_PER_ROW;
        self.bits[start..start + WORDS_PER_ROW]
            .iter()
            .map(|w| w.count_ones() as usize)
            .sum()
    }

    pub fn words_per_row(&self) -> usize {
        WORDS_PER_ROW
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_holdem_combo_has_1225_disjoint_opponent_combos() {
        let matrix = BlockerMatrix::build();
        assert_eq!(matrix.words_per_row(), 21);
        for i in [0usize, 1, 100, 700, 1325] {
            assert_eq!(matrix.compatible_count(i), 1225);
            assert!(!matrix.compatible(i, i));
        }
    }

    #[test]
    fn compatibility_is_symmetric() {
        let matrix = BlockerMatrix::build();
        for (a, b) in [(0, 100), (12, 1200), (455, 900)] {
            assert_eq!(matrix.compatible(a, b), matrix.compatible(b, a));
        }
    }
}

use crate::blockers::BlockerMatrix;
use crate::cards::COMBO_COUNT;
use crate::range::ComboRange;

pub fn joint_compatible_pair_count(hero_combo: usize, blockers: &BlockerMatrix) -> usize {
    assert!(hero_combo < COMBO_COUNT);
    let mut count = 0usize;
    for a in 0..COMBO_COUNT {
        if !blockers.compatible(hero_combo, a) {
            continue;
        }
        for b in 0..COMBO_COUNT {
            if blockers.compatible(hero_combo, b) && blockers.compatible(a, b) {
                count += 1;
            }
        }
    }
    count
}

pub fn joint_normalizer(
    hero_combo: usize,
    first: &ComboRange,
    second: &ComboRange,
    blockers: &BlockerMatrix,
) -> f64 {
    assert!(hero_combo < COMBO_COUNT);
    let wa = first.weights();
    let wb = second.weights();
    let mut z = 0.0;
    for a in 0..COMBO_COUNT {
        if wa[a] <= 0.0 || !blockers.compatible(hero_combo, a) {
            continue;
        }
        for b in 0..COMBO_COUNT {
            if wb[b] <= 0.0 {
                continue;
            }
            if blockers.compatible(hero_combo, b) && blockers.compatible(a, b) {
                z += wa[a] * wb[b];
            }
        }
    }
    z
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_three_player_pair_count_after_hero_cards() {
        let blockers = BlockerMatrix::build();
        assert_eq!(joint_compatible_pair_count(0, &blockers), 1_381_800);
    }

    #[test]
    fn uniform_joint_normalizer_matches_combinatorics() {
        let blockers = BlockerMatrix::build();
        let r = ComboRange::uniform();
        let z = joint_normalizer(0, &r, &r, &blockers);
        let expected = 1_381_800.0 / (COMBO_COUNT as f64 * COMBO_COUNT as f64);
        assert!((z - expected).abs() < 1e-10);
    }
}

import unittest

from solver.postflop_provider import (
    HandContinuationEV,
    PostflopSolveResult,
    validate_result,
)
from solver.three_max_batch_cfr import build_frozen_corpus


class FrozenCorpusTests(unittest.TestCase):
    def test_corpus_is_deterministic_for_same_seed(self):
        a = build_frozen_corpus(samples_per_hand=1, seed=123)
        b = build_frozen_corpus(samples_per_hand=1, seed=123)
        self.assertEqual(a, b)

    def test_corpus_changes_with_seed(self):
        a = build_frozen_corpus(samples_per_hand=1, seed=123)
        b = build_frozen_corpus(samples_per_hand=1, seed=124)
        self.assertNotEqual(a, b)


class PostflopProviderTests(unittest.TestCase):
    def test_measured_result_requires_exploitability(self):
        result = PostflopSolveResult(
            node_id="x",
            provider="local",
            provider_version="1",
            exploitability_pct_pot=None,
            status="POSTFLOP_SOLVED_MEASURED",
            hand_evs=(HandContinuationEV("A5s", 0.2),),
            assumptions_hash="abc",
        )
        with self.assertRaises(ValueError):
            validate_result(result)

    def test_valid_measured_result(self):
        result = PostflopSolveResult(
            node_id="x",
            provider="local",
            provider_version="1",
            exploitability_pct_pot=0.25,
            status="POSTFLOP_SOLVED_MEASURED",
            hand_evs=(HandContinuationEV("A5s", 0.2),),
            assumptions_hash="abc",
        )
        validate_result(result)


if __name__ == "__main__":
    unittest.main()

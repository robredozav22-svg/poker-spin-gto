import unittest

from solver.hands import HAND_CLASSES
from solver.three_max_external_mccfr import solve_three_max_pushfold_external_stratified


class ExternalThreeMaxTests(unittest.TestCase):
    def test_one_sweep_covers_all_strategy_nodes(self):
        result = solve_three_max_pushfold_external_stratified(stack_bb=8.0, sweeps=1, seed=19)
        self.assertEqual(result.chance_samples, 3 * 169)
        self.assertEqual(len(result.strategies), 6)
        self.assertIn("BTN_ROOT", result.strategies)
        for chart in result.strategies.values():
            self.assertEqual(len(chart), 169)
            for hand in HAND_CLASSES:
                self.assertAlmostEqual(sum(chart[hand].values()), 1.0, places=9)


if __name__ == "__main__":
    unittest.main()

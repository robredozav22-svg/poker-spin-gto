import unittest

from solver.dcfr import DiscountedRegretNode
from solver.hands import HAND_CLASSES
from solver.three_max_external_dcfr import solve_three_max_pushfold_external_dcfr


class DcfrNodeTests(unittest.TestCase):
    def test_better_action_dominates(self):
        node = DiscountedRegretNode(("fold", "jam"))
        for t in range(1, 101):
            strategy = node.current_strategy()
            values = {"fold": 0.0, "jam": 1.0}
            node_value = sum(strategy[a] * values[a] for a in strategy)
            node.add_regrets(values, node_value, iteration=t)
            node.accumulate_strategy(iteration=t)
        self.assertGreater(node.average_strategy()["jam"], 0.90)

    def test_regret_floor_never_negative(self):
        node = DiscountedRegretNode(("a", "b"), regret_floor=True)
        node.add_regrets({"a": -1.0, "b": 1.0}, 0.0, iteration=1)
        self.assertGreaterEqual(node.regret_sum["a"], 0.0)
        self.assertGreaterEqual(node.regret_sum["b"], 0.0)


class ThreeMaxDcfrTests(unittest.TestCase):
    def test_small_run_returns_normalized_strategy(self):
        result = solve_three_max_pushfold_external_dcfr(8.0, sweeps=1, seed=23)
        self.assertEqual(result.chance_samples, 3 * 169)
        self.assertEqual(len(result.strategies), 6)
        for chart in result.strategies.values():
            self.assertEqual(len(chart), 169)
            for hand in HAND_CLASSES:
                self.assertAlmostEqual(sum(chart[hand].values()), 1.0, places=9)


if __name__ == "__main__":
    unittest.main()

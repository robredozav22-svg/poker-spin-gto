import unittest

from solver.equity import concrete_equity_sampled
from solver.evaluator import evaluate_five, evaluate_seven
from solver.hands import HAND_CLASSES, class_combo_count, expand_hand_class
from solver.model import Action, DecisionNodeKey, GameConfig, Mode, PayoutProfile, TreeAction
from solver.pushfold_hu import solve_hu_pushfold
from solver.regret import RegretNode, expected_value


class HandModelTests(unittest.TestCase):
    def test_169_classes_and_1326_combos(self):
        self.assertEqual(len(HAND_CLASSES), 169)
        self.assertEqual(sum(class_combo_count(h) for h in HAND_CLASSES), 1326)

    def test_combo_counts(self):
        self.assertEqual(len(expand_hand_class("AA")), 6)
        self.assertEqual(len(expand_hand_class("AKs")), 4)
        self.assertEqual(len(expand_hand_class("AKo")), 12)


class EvaluatorTests(unittest.TestCase):
    def test_hand_category_order(self):
        straight_flush = evaluate_five(("As", "Ks", "Qs", "Js", "Ts"))
        quads = evaluate_five(("Ah", "Ad", "Ac", "As", "Kd"))
        full_house = evaluate_five(("Kh", "Kd", "Kc", "2s", "2d"))
        self.assertGreater(straight_flush, quads)
        self.assertGreater(quads, full_house)

    def test_wheel_straight(self):
        wheel = evaluate_five(("As", "2d", "3c", "4h", "5s"))
        six_high = evaluate_five(("2s", "3d", "4c", "5h", "6s"))
        self.assertEqual(wheel[:2], (4, 5))
        self.assertGreater(six_high, wheel)

    def test_best_of_seven(self):
        rank = evaluate_seven(("As", "Ks", "Qs", "Js", "Ts", "2d", "3c"))
        self.assertEqual(rank[:2], (8, 14))

    def test_sampled_equity_is_self_generated(self):
        result = concrete_equity_sampled(("As", "Ah"), ("Ks", "Kh"), boards=800, seed=7)
        self.assertGreater(result.equity, 0.70)
        self.assertEqual(result.total, 800)


class ModelTests(unittest.TestCase):
    def test_canonical_node_key_preserves_history_and_stacks(self):
        cfg = GameConfig(Mode.THREE_MAX, PayoutProfile.WTA, (15.0, 15.0, 15.0))
        key = DecisionNodeKey(
            mode=cfg.mode,
            payout_profile=cfg.payout_profile,
            stacks_bb=cfg.stacks_bb,
            hero="BB",
            history=(
                TreeAction("BTN", Action.RAISE, 2.0),
                TreeAction("SB", Action.CALL),
            ),
        )
        self.assertEqual(key.canonical(), "3max|wta|15-15-15|BTN_RAISE_2>SB_CALL|BB")


class RegretTests(unittest.TestCase):
    def test_regret_matching_moves_to_better_action(self):
        node = RegretNode(("fold", "jam"))
        for _ in range(100):
            strategy = node.accumulate_strategy()
            values = {"fold": 0.0, "jam": 1.0}
            node.add_regrets(values, expected_value(strategy, values))
        self.assertGreater(node.average_strategy()["jam"], 0.95)

    def test_pushfold_refuses_incomplete_equity_table(self):
        with self.assertRaises(ValueError):
            solve_hu_pushfold({}, stack_bb=2.0, iterations=1)


if __name__ == "__main__":
    unittest.main()

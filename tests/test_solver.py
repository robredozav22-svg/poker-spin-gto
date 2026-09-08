import unittest

from solver.continuation import (
    ContinuationStateKey,
    ContinuationValue,
    ContinuationValueTable,
    MissingContinuationValue,
)
from solver.convergence import combo_weighted_action_frequency, compare_action
from solver.equity import concrete_equity_sampled
from solver.evaluator import evaluate_five, evaluate_seven
from solver.hands import HAND_CLASSES, class_combo_count, combo_to_class, expand_hand_class
from solver.hu_mccfr import solve_hu_pushfold_chance_sampled
from solver.model import Action, DecisionNodeKey, GameConfig, Mode, PayoutProfile, TreeAction
from solver.pots import settle_pots
from solver.pushfold_hu import solve_hu_pushfold
from solver.regret import RegretNode, expected_value
from solver.three_max_mccfr import solve_three_max_pushfold_chance_sampled, terminal_payoff


class HandModelTests(unittest.TestCase):
    def test_169_classes_and_1326_combos(self):
        self.assertEqual(len(HAND_CLASSES), 169)
        self.assertEqual(sum(class_combo_count(h) for h in HAND_CLASSES), 1326)

    def test_combo_counts(self):
        self.assertEqual(len(expand_hand_class("AA")), 6)
        self.assertEqual(len(expand_hand_class("AKs")), 4)
        self.assertEqual(len(expand_hand_class("AKo")), 12)

    def test_combo_to_class(self):
        self.assertEqual(combo_to_class(("As", "Ah")), "AA")
        self.assertEqual(combo_to_class(("As", "Ks")), "AKs")
        self.assertEqual(combo_to_class(("Kd", "As")), "AKo")


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


class PotSettlementTests(unittest.TestCase):
    def test_three_way_equal_allin_is_zero_sum(self):
        payoff = settle_pots(
            {"BTN": 10.0, "SB": 10.0, "BB": 10.0},
            {"BTN": False, "SB": False, "BB": False},
            {"BTN": (1,), "SB": (3,), "BB": (2,)},
        )
        self.assertEqual(payoff, {"BTN": -10.0, "SB": 20.0, "BB": -10.0})
        self.assertAlmostEqual(sum(payoff.values()), 0.0)

    def test_asymmetric_side_pot(self):
        payoff = settle_pots(
            {"BTN": 5.0, "SB": 10.0, "BB": 10.0},
            {"BTN": False, "SB": False, "BB": False},
            {"BTN": (3,), "SB": (1,), "BB": (2,)},
        )
        self.assertEqual(payoff, {"BTN": 10.0, "SB": -10.0, "BB": 0.0})
        self.assertAlmostEqual(sum(payoff.values()), 0.0)

    def test_folded_chips_stay_in_pot(self):
        payoff = settle_pots(
            {"BTN": 2.0, "SB": 2.0, "BB": 1.0},
            {"BTN": True, "SB": False, "BB": True},
            {"SB": (1,)},
        )
        self.assertEqual(payoff, {"BTN": -2.0, "SB": 3.0, "BB": -1.0})
        self.assertAlmostEqual(sum(payoff.values()), 0.0)


class ThreeMaxTerminalTests(unittest.TestCase):
    def setUp(self):
        self.ranks = {"BTN": (3,), "SB": (2,), "BB": (1,)}

    def test_everybody_folds_to_bb(self):
        payoff = terminal_payoff(("BTN:fold", "SB:fold"), 8.0, self.ranks)
        self.assertEqual(payoff, {"BTN": 0.0, "SB": -0.5, "BB": 0.5})

    def test_btn_jam_both_fold(self):
        payoff = terminal_payoff(("BTN:jam", "SB:fold", "BB:fold"), 8.0, self.ranks)
        self.assertEqual(payoff, {"BTN": 1.5, "SB": -0.5, "BB": -1.0})

    def test_btn_beats_bb_with_dead_sb_blind(self):
        payoff = terminal_payoff(("BTN:jam", "SB:fold", "BB:call"), 8.0, self.ranks)
        self.assertEqual(payoff, {"BTN": 8.5, "SB": -0.5, "BB": -8.0})
        self.assertAlmostEqual(sum(payoff.values()), 0.0)

    def test_three_way_showdown(self):
        payoff = terminal_payoff(("BTN:jam", "SB:call", "BB:call"), 8.0, self.ranks)
        self.assertEqual(payoff, {"BTN": 16.0, "SB": -8.0, "BB": -8.0})
        self.assertAlmostEqual(sum(payoff.values()), 0.0)


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


class ContinuationTests(unittest.TestCase):
    def _key(self):
        node = DecisionNodeKey(
            mode=Mode.THREE_MAX,
            payout_profile=PayoutProfile.WTA,
            stacks_bb=(15.0, 15.0, 15.0),
            hero="BB",
            history=(
                TreeAction("BTN", Action.RAISE, 2.0),
                TreeAction("SB", Action.CALL),
                TreeAction("BB", Action.CALL),
            ),
        )
        return ContinuationStateKey(node=node, hand_class="A5s", pot_bb=6.0, spr=2.1667)

    def test_missing_continuation_is_fatal(self):
        table = ContinuationValueTable()
        with self.assertRaises(MissingContinuationValue):
            table.require(self._key())

    def test_continuation_requires_provenance(self):
        table = ContinuationValueTable()
        with self.assertRaises(ValueError):
            table.put(
                self._key(),
                ContinuationValue(ev_bb=0.25, source="", status="CONTINUATION_APPROX"),
            )

    def test_continuation_round_trip(self):
        table = ContinuationValueTable()
        value = ContinuationValue(
            ev_bb=0.25,
            source="POSTFLOP_SUBGAME_TEST",
            status="CONTINUATION_APPROX",
            samples=1000,
            abstraction="toy",
        )
        table.put(self._key(), value)
        self.assertEqual(table.require(self._key()), value)


class ConvergenceTests(unittest.TestCase):
    def test_identical_chart_has_zero_distance(self):
        chart = {h: {"fold": 0.25, "jam": 0.75} for h in HAND_CLASSES}
        distance = compare_action("BTN_ROOT", chart, chart, "jam")
        self.assertAlmostEqual(distance.combo_weighted_mae_pct, 0.0)
        self.assertAlmostEqual(distance.max_hand_delta_pct, 0.0)

    def test_combo_weighted_frequency(self):
        chart = {h: {"fold": 0.0, "jam": 1.0} for h in HAND_CLASSES}
        self.assertAlmostEqual(combo_weighted_action_frequency(chart, "jam"), 1.0)


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

    def test_chance_sampled_solver_is_self_contained(self):
        result = solve_hu_pushfold_chance_sampled(stack_bb=2.0, iterations=300, seed=9)
        self.assertEqual(len(result.btn), 169)
        self.assertEqual(len(result.bb), 169)
        for hand in HAND_CLASSES:
            self.assertAlmostEqual(sum(result.btn[hand].values()), 1.0, places=9)
            self.assertAlmostEqual(sum(result.bb[hand].values()), 1.0, places=9)

    def test_three_max_solver_returns_all_public_nodes(self):
        result = solve_three_max_pushfold_chance_sampled(stack_bb=8.0, iterations=60, seed=17)
        self.assertEqual(len(result.strategies), 6)
        self.assertIn("BTN_ROOT", result.strategies)
        for chart in result.strategies.values():
            self.assertEqual(len(chart), 169)
            for strategy in chart.values():
                self.assertAlmostEqual(sum(strategy.values()), 1.0, places=9)


if __name__ == "__main__":
    unittest.main()

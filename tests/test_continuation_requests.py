import unittest

from solver.continuation_requests import fifteen_bb_core_postflop_requests


class ContinuationRequestTests(unittest.TestCase):
    def test_core_15bb_subgames_have_expected_pots_and_spr(self):
        rows = {r.id: r for r in fifteen_bb_core_postflop_requests()}
        self.assertEqual(set(rows), {
            "15_BTN_R2_SB_FOLD_BB_CALL",
            "15_BTN_R2_SB_CALL_BB_CALL",
            "15_BTN_FOLD_SB_LIMP_BB_CHECK",
            "15_BTN_FOLD_SB_R3_BB_CALL",
        })
        self.assertAlmostEqual(rows["15_BTN_R2_SB_FOLD_BB_CALL"].pot_bb, 4.5)
        self.assertAlmostEqual(rows["15_BTN_R2_SB_FOLD_BB_CALL"].spr, 13.0 / 4.5)
        self.assertAlmostEqual(rows["15_BTN_R2_SB_CALL_BB_CALL"].pot_bb, 6.0)
        self.assertAlmostEqual(rows["15_BTN_R2_SB_CALL_BB_CALL"].spr, 13.0 / 6.0)
        self.assertAlmostEqual(rows["15_BTN_FOLD_SB_LIMP_BB_CHECK"].spr, 7.0)
        self.assertAlmostEqual(rows["15_BTN_FOLD_SB_R3_BB_CALL"].spr, 2.0)


if __name__ == "__main__":
    unittest.main()

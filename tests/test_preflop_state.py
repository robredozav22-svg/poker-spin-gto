import unittest

from solver.preflop_state import derive_state


class PreflopStateTests(unittest.TestCase):
    def test_btn_r2_sb_fold_bb_call(self):
        s = derive_state(15, ["BTN_RAISE_TO_2", "SB_FOLD", "BB_CALL"])
        self.assertAlmostEqual(s.pot_bb, 4.5)
        self.assertAlmostEqual(s.behind("BTN"), 13.0)
        self.assertAlmostEqual(s.behind("BB"), 13.0)
        self.assertAlmostEqual(s.spr(), 13.0 / 4.5)

    def test_btn_r2_sb_call_bb_call(self):
        s = derive_state(15, ["BTN_RAISE_TO_2", "SB_CALL", "BB_CALL"])
        self.assertAlmostEqual(s.pot_bb, 6.0)
        self.assertAlmostEqual(s.effective_postflop_stack(), 13.0)
        self.assertAlmostEqual(s.spr(), 13.0 / 6.0)

    def test_btn_fold_sb_limp_bb_check(self):
        s = derive_state(15, ["BTN_FOLD", "SB_LIMP", "BB_CHECK"])
        self.assertAlmostEqual(s.pot_bb, 2.0)
        self.assertAlmostEqual(s.effective_postflop_stack(), 14.0)
        self.assertAlmostEqual(s.spr(), 7.0)

    def test_btn_fold_sb_r3_bb_call(self):
        s = derive_state(15, ["BTN_FOLD", "SB_RAISE_TO_3", "BB_CALL"])
        self.assertAlmostEqual(s.pot_bb, 6.0)
        self.assertAlmostEqual(s.effective_postflop_stack(), 12.0)
        self.assertAlmostEqual(s.spr(), 2.0)


if __name__ == "__main__":
    unittest.main()

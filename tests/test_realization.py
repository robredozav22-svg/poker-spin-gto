import unittest

from solver.realization import (
    Initiative,
    MissingRealizationCoefficient,
    PositionRole,
    RealizationContext,
    RealizationObservation,
    fit_realization_observations,
    normalized_realized_shares,
    spr_band,
)


class RealizationTests(unittest.TestCase):
    def test_fit_requires_exact_context_hand_support(self):
        ctx = RealizationContext(PositionRole.IP, Initiative.AGGRESSOR, "LE4", 2)
        model = fit_realization_observations([
            RealizationObservation(ctx, "A5s", raw_equity=0.50, realized_pot_share=0.55),
            RealizationObservation(ctx, "A5s", raw_equity=0.50, realized_pot_share=0.60),
        ])
        self.assertAlmostEqual(model.require(ctx, "A5s").value, 1.15)
        with self.assertRaises(MissingRealizationCoefficient):
            model.require(ctx, "KQo")

    def test_normalized_shares_conserve_pot(self):
        shares = normalized_realized_shares(
            {"IP": 0.45, "OOP": 0.55},
            {"IP": 1.10, "OOP": 0.90},
        )
        self.assertAlmostEqual(sum(shares.values()), 1.0)
        self.assertGreater(shares["IP"], 0.45)

    def test_spr_bands_cover_spin_targets(self):
        self.assertEqual(spr_band(2.0), "LE2_5")
        self.assertEqual(spr_band(13.0 / 4.5), "LE4")
        self.assertEqual(spr_band(7.0), "LE8")


if __name__ == "__main__":
    unittest.main()

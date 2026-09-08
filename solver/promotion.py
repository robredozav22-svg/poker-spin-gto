from __future__ import annotations

from dataclasses import dataclass


@dataclass(frozen=True)
class PromotionDecision:
    passed: bool
    reasons: tuple[str, ...]


def evaluate_stability_report(
    report: dict,
    *,
    max_root_delta_pp: float = 0.50,
    max_node_combo_mae_pp: float = 2.00,
) -> PromotionDecision:
    """Gate solver output on numerical stability only.

    Passing this gate does NOT mean full-game GTO correctness. It means only
    that independent runs of the same declared abstraction are sufficiently
    stable to move to the next validation stage.
    """
    reasons: list[str] = []
    root_delta = float(report["btn_root_jam_delta_pct_points"])
    node_mae = float(report["max_combo_weighted_mae_pct"])
    if root_delta > max_root_delta_pp:
        reasons.append(
            f"root strategy unstable: {root_delta:.3f}pp > {max_root_delta_pp:.3f}pp"
        )
    if node_mae > max_node_combo_mae_pp:
        reasons.append(
            f"node strategy unstable: {node_mae:.3f}pp > {max_node_combo_mae_pp:.3f}pp"
        )
    return PromotionDecision(passed=not reasons, reasons=tuple(reasons))

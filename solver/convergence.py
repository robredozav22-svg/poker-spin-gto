from __future__ import annotations

from dataclasses import dataclass

from .hands import HAND_CLASSES, class_combo_count


@dataclass(frozen=True)
class StrategyDistance:
    node: str
    action: str
    combo_weighted_mae_pct: float
    max_hand_delta_pct: float
    max_hand: str


def combo_weighted_action_frequency(
    chart: dict[str, dict[str, float]], action: str
) -> float:
    """Return combo-weighted action frequency in [0, 1]."""
    total = sum(class_combo_count(h) for h in HAND_CLASSES)
    if total != 1326:
        raise AssertionError(f"unexpected combo total: {total}")
    weighted = 0.0
    for hand in HAND_CLASSES:
        if hand not in chart:
            raise ValueError(f"missing hand {hand}")
        strategy = chart[hand]
        if action not in strategy:
            raise ValueError(f"missing action {action} for {hand}")
        weighted += class_combo_count(hand) * float(strategy[action])
    return weighted / total


def compare_action(
    node: str,
    chart_a: dict[str, dict[str, float]],
    chart_b: dict[str, dict[str, float]],
    action: str,
) -> StrategyDistance:
    total = 1326
    weighted_abs = 0.0
    max_delta = -1.0
    max_hand = ""
    for hand in HAND_CLASSES:
        if hand not in chart_a or hand not in chart_b:
            raise ValueError(f"missing hand {hand} in comparison")
        if action not in chart_a[hand] or action not in chart_b[hand]:
            raise ValueError(f"missing action {action} for {hand}")
        delta = abs(float(chart_a[hand][action]) - float(chart_b[hand][action]))
        weighted_abs += class_combo_count(hand) * delta
        if delta > max_delta:
            max_delta = delta
            max_hand = hand
    return StrategyDistance(
        node=node,
        action=action,
        combo_weighted_mae_pct=100.0 * weighted_abs / total,
        max_hand_delta_pct=100.0 * max_delta,
        max_hand=max_hand,
    )


def compare_strategy_sets(
    strategies_a: dict[str, dict[str, dict[str, float]]],
    strategies_b: dict[str, dict[str, dict[str, float]]],
) -> list[StrategyDistance]:
    """Compare two complete solver outputs node-by-node/action-by-action.

    This measures numerical stability between independent runs. It is not
    exploitability and does not prove correctness of the game abstraction.
    """
    if set(strategies_a) != set(strategies_b):
        raise ValueError("strategy sets contain different public nodes")

    out: list[StrategyDistance] = []
    for node in sorted(strategies_a):
        a = strategies_a[node]
        b = strategies_b[node]
        first = HAND_CLASSES[0]
        actions_a = set(a[first])
        actions_b = set(b[first])
        if actions_a != actions_b:
            raise ValueError(f"action mismatch at {node}")
        for action in sorted(actions_a):
            out.append(compare_action(node, a, b, action))
    return out

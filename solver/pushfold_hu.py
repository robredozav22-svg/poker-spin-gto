from __future__ import annotations

from functools import lru_cache

from .hands import HAND_CLASSES, expand_hand_class
from .regret import RegretNode, expected_value


@lru_cache(maxsize=None)
def compatible_pair_count(a: str, b: str) -> int:
    count = 0
    for ca in expand_hand_class(a):
        for cb in expand_hand_class(b):
            if not (set(ca) & set(cb)):
                count += 1
    return count


def validate_equity_table(table: dict[str, dict[str, float]]) -> None:
    missing_rows = [h for h in HAND_CLASSES if h not in table]
    if missing_rows:
        raise ValueError(f"Equity table missing rows, first: {missing_rows[:5]}")
    for h in HAND_CLASSES:
        missing = [v for v in HAND_CLASSES if v not in table[h]]
        if missing:
            raise ValueError(f"Equity row {h} missing columns, first: {missing[:5]}")
        for v, eq in table[h].items():
            if not 0.0 <= float(eq) <= 1.0:
                raise ValueError(f"Invalid equity {h} vs {v}: {eq}")


def _villain_class_weights(hero_class: str) -> dict[str, float]:
    raw = {v: compatible_pair_count(hero_class, v) for v in HAND_CLASSES}
    total = sum(raw.values())
    return {v: n / total for v, n in raw.items() if n > 0}


def solve_hu_pushfold(
    equity_table: dict[str, dict[str, float]],
    stack_bb: float,
    iterations: int = 20_000,
) -> dict[str, dict[str, dict[str, float]]]:
    """Solve the HU SB/BTN jam-or-fold vs BB call-or-fold abstraction.

    Conventions:
    - starting stacks are `stack_bb` before posting blinds;
    - BTN is the small blind (0.5bb), BB posts 1bb;
    - BTN fold payoff = -0.5bb;
    - BTN jam and BB fold payoff = +1bb;
    - called all-in payoff for either player is (2*equity-1)*stack_bb.

    This is a *push/fold abstraction*. It is not a valid replacement for a
    full Spin tree when limp/minraise/postflop actions are available.
    """
    if stack_bb <= 1:
        raise ValueError("stack_bb must be > 1 for this convention")
    if iterations <= 0:
        raise ValueError("iterations must be positive")
    validate_equity_table(equity_table)

    btn = {h: RegretNode(("fold", "jam")) for h in HAND_CLASSES}
    bb = {h: RegretNode(("fold", "call")) for h in HAND_CLASSES}
    class_weights = {h: _villain_class_weights(h) for h in HAND_CLASSES}

    for _ in range(iterations):
        btn_strategy = {h: btn[h].accumulate_strategy() for h in HAND_CLASSES}
        bb_strategy = {h: bb[h].accumulate_strategy() for h in HAND_CLASSES}

        # BTN regrets by private hand class.
        for h in HAND_CLASSES:
            jam_ev = 0.0
            for v, p_v in class_weights[h].items():
                call_p = bb_strategy[v]["call"]
                eq = float(equity_table[h][v])
                called = (2.0 * eq - 1.0) * stack_bb
                jam_ev += p_v * ((1.0 - call_p) * 1.0 + call_p * called)
            values = {"fold": -0.5, "jam": jam_ev}
            strategy = btn_strategy[h]
            node_ev = expected_value(strategy, values)
            btn[h].add_regrets(values, node_ev)

        # BB regrets by private hand class, conditional on facing a jam.
        for v in HAND_CLASSES:
            numerator = 0.0
            denominator = 0.0
            for h in HAND_CLASSES:
                compat = compatible_pair_count(h, v)
                if compat <= 0:
                    continue
                shove_p = btn_strategy[h]["jam"]
                weight = compat * shove_p
                if weight <= 0:
                    continue
                hero_eq = float(equity_table[h][v])
                bb_eq = 1.0 - hero_eq
                numerator += weight * ((2.0 * bb_eq - 1.0) * stack_bb)
                denominator += weight

            call_ev = numerator / denominator if denominator > 0 else -1.0
            values = {"fold": -1.0, "call": call_ev}
            strategy = bb_strategy[v]
            node_ev = expected_value(strategy, values)
            bb[v].add_regrets(values, node_ev)

    return {
        "BTN": {h: btn[h].average_strategy() for h in HAND_CLASSES},
        "BB": {h: bb[h].average_strategy() for h in HAND_CLASSES},
        "meta": {
            "model": "HU_PUSH_FOLD_ABSTRACTION_V1",
            "stack_bb": stack_bb,
            "iterations": iterations,
            "exact_tree": False,
        },
    }

from __future__ import annotations

import argparse
import json
from pathlib import Path

from .convergence import combo_weighted_action_frequency, compare_strategy_sets
from .three_max_mccfr import solve_three_max_pushfold_chance_sampled


def main() -> None:
    parser = argparse.ArgumentParser(description="Measure 3-max solver stability across independent runs")
    parser.add_argument("--stack", type=float, default=8.0)
    parser.add_argument("--iterations-a", type=int, default=100_000)
    parser.add_argument("--iterations-b", type=int, default=300_000)
    parser.add_argument("--seed-a", type=int, default=20260910)
    parser.add_argument("--seed-b", type=int, default=20260911)
    parser.add_argument("--output", required=True)
    args = parser.parse_args()

    a = solve_three_max_pushfold_chance_sampled(args.stack, args.iterations_a, args.seed_a)
    b = solve_three_max_pushfold_chance_sampled(args.stack, args.iterations_b, args.seed_b)

    distances = compare_strategy_sets(a.strategies, b.strategies)
    root_a = combo_weighted_action_frequency(a.strategies["BTN_ROOT"], "jam")
    root_b = combo_weighted_action_frequency(b.strategies["BTN_ROOT"], "jam")

    payload = {
        "schema_version": "1.0",
        "status": "SOLVER_APPROX_DIAGNOSTIC",
        "model": "THREE_MAX_CHANCE_SAMPLED_PUSH_FOLD_V1",
        "stack_bb": args.stack,
        "run_a": {"iterations": args.iterations_a, "seed": args.seed_a},
        "run_b": {"iterations": args.iterations_b, "seed": args.seed_b},
        "btn_root_jam_pct": {"a": 100.0 * root_a, "b": 100.0 * root_b},
        "btn_root_jam_delta_pct_points": 100.0 * abs(root_a - root_b),
        "max_combo_weighted_mae_pct": max(d.combo_weighted_mae_pct for d in distances),
        "max_hand_delta_pct": max(d.max_hand_delta_pct for d in distances),
        "distances": [d.__dict__ for d in distances],
        "interpretation": (
            "Numerical stability inside the restricted 3-max push/fold abstraction only. "
            "Low distance does not validate missing limp/minraise/postflop branches."
        ),
    }

    out = Path(args.output)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(payload, ensure_ascii=False, indent=2, sort_keys=True), encoding="utf-8")
    print(json.dumps({k: payload[k] for k in [
        "btn_root_jam_pct",
        "btn_root_jam_delta_pct_points",
        "max_combo_weighted_mae_pct",
        "max_hand_delta_pct",
    ]}, indent=2))


if __name__ == "__main__":
    main()

from __future__ import annotations

import argparse
import json
from pathlib import Path

from .convergence import compare_action, combo_weighted_action_frequency
from .three_max_batch_cfr import solve_three_max_pushfold_batch_cfr


def main() -> None:
    p = argparse.ArgumentParser()
    p.add_argument("--stack", type=float, required=True)
    p.add_argument("--sweeps-a", type=int, default=200)
    p.add_argument("--sweeps-b", type=int, default=600)
    p.add_argument("--samples-per-hand", type=int, default=8)
    p.add_argument("--corpus-seed-a", type=int, default=20260916)
    p.add_argument("--corpus-seed-b", type=int, default=20260917)
    p.add_argument("--output", required=True)
    args = p.parse_args()

    a = solve_three_max_pushfold_batch_cfr(
        args.stack, args.sweeps_a, args.corpus_seed_a, args.samples_per_hand
    )
    b = solve_three_max_pushfold_batch_cfr(
        args.stack, args.sweeps_b, args.corpus_seed_b, args.samples_per_hand
    )

    distances = []
    max_mae = 0.0
    max_hand_delta = 0.0
    for node, chart_a in a.strategies.items():
        chart_b = b.strategies[node]
        for action in next(iter(chart_a.values())).keys():
            d = compare_action(node, chart_a, chart_b, action)
            distances.append(d.__dict__)
            max_mae = max(max_mae, d.combo_weighted_mae_pct)
            max_hand_delta = max(max_hand_delta, d.max_hand_delta_pct)

    root_a = 100.0 * combo_weighted_action_frequency(a.strategies["BTN_ROOT"], "jam")
    root_b = 100.0 * combo_weighted_action_frequency(b.strategies["BTN_ROOT"], "jam")
    root_delta = abs(root_a - root_b)
    gate = {
        "root_delta_limit_pct_points": 0.50,
        "max_node_combo_weighted_mae_limit_pct": 2.00,
    }
    passed = root_delta <= 0.50 and max_mae <= 2.00

    payload = {
        "schema_version": "1.0",
        "status": "SOLVER_APPROX_DIAGNOSTIC",
        "model": "THREE_MAX_FIXED_CORPUS_BATCH_CFR_V1",
        "stack_bb": args.stack,
        "run_a": {
            "sweeps": args.sweeps_a,
            "corpus_seed": args.corpus_seed_a,
            "samples_per_hand": args.samples_per_hand,
        },
        "run_b": {
            "sweeps": args.sweeps_b,
            "corpus_seed": args.corpus_seed_b,
            "samples_per_hand": args.samples_per_hand,
        },
        "btn_root_jam_pct": {"a": root_a, "b": root_b},
        "btn_root_jam_delta_pct_points": root_delta,
        "max_combo_weighted_mae_pct": max_mae,
        "max_hand_delta_pct": max_hand_delta,
        "distances": distances,
        "stability_gate": {
            "passed": passed,
            "limits": gate,
            "interpretation": "Numerical stability across independent frozen chance corpora only; not full Spin GTO certification.",
        },
    }
    out = Path(args.output)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(payload, indent=2, sort_keys=True), encoding="utf-8")
    print(json.dumps({k: payload[k] for k in ["btn_root_jam_pct","btn_root_jam_delta_pct_points","max_combo_weighted_mae_pct","stability_gate"]}, indent=2))


if __name__ == "__main__":
    main()

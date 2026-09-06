from __future__ import annotations

import argparse
import json
from pathlib import Path

from .convergence import combo_weighted_action_frequency, compare_strategy_sets
from .three_max_external_mccfr import solve_three_max_pushfold_external_stratified


def main() -> None:
    parser = argparse.ArgumentParser(description="Measure stratified external-sampling 3-max stability")
    parser.add_argument("--stack", type=float, default=8.0)
    parser.add_argument("--sweeps-a", type=int, default=200)
    parser.add_argument("--sweeps-b", type=int, default=600)
    parser.add_argument("--seed-a", type=int, default=20260912)
    parser.add_argument("--seed-b", type=int, default=20260913)
    parser.add_argument("--output", required=True)
    args = parser.parse_args()

    a = solve_three_max_pushfold_external_stratified(args.stack, args.sweeps_a, args.seed_a)
    b = solve_three_max_pushfold_external_stratified(args.stack, args.sweeps_b, args.seed_b)
    distances = compare_strategy_sets(a.strategies, b.strategies)
    ja = 100.0 * combo_weighted_action_frequency(a.strategies["BTN_ROOT"], "jam")
    jb = 100.0 * combo_weighted_action_frequency(b.strategies["BTN_ROOT"], "jam")

    payload = {
        "schema_version": "1.0",
        "status": "SOLVER_APPROX_DIAGNOSTIC",
        "model": "THREE_MAX_EXTERNAL_STRATIFIED_PUSH_FOLD_V1",
        "stack_bb": args.stack,
        "run_a": {"sweeps": args.sweeps_a, "seed": args.seed_a, "chance_samples": a.chance_samples},
        "run_b": {"sweeps": args.sweeps_b, "seed": args.seed_b, "chance_samples": b.chance_samples},
        "btn_root_jam_pct": {"a": ja, "b": jb},
        "btn_root_jam_delta_pct_points": abs(ja - jb),
        "max_combo_weighted_mae_pct": max(d.combo_weighted_mae_pct for d in distances),
        "max_hand_delta_pct": max(d.max_hand_delta_pct for d in distances),
        "distances": [d.__dict__ for d in distances],
        "interpretation": "Stability diagnostic inside the restricted push/fold abstraction; not full-game exploitability.",
    }
    out = Path(args.output)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(payload, ensure_ascii=False, indent=2, sort_keys=True), encoding="utf-8")
    print(json.dumps({k: payload[k] for k in ["btn_root_jam_pct","btn_root_jam_delta_pct_points","max_combo_weighted_mae_pct","max_hand_delta_pct"]}, indent=2))


if __name__ == "__main__":
    main()

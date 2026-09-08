from __future__ import annotations

import argparse
import json
from pathlib import Path

from .convergence import combo_weighted_action_frequency, compare_strategy_sets
from .promotion import evaluate_stability_report
from .three_max_external_dcfr import solve_three_max_pushfold_external_dcfr


def main() -> None:
    p = argparse.ArgumentParser(description="Measure stratified DCFR 3-max stability")
    p.add_argument("--stack", type=float, default=8.0)
    p.add_argument("--sweeps-a", type=int, default=300)
    p.add_argument("--sweeps-b", type=int, default=900)
    p.add_argument("--seed-a", type=int, default=20260914)
    p.add_argument("--seed-b", type=int, default=20260915)
    p.add_argument("--regret-floor", action="store_true")
    p.add_argument("--output", required=True)
    args = p.parse_args()

    a = solve_three_max_pushfold_external_dcfr(
        args.stack, args.sweeps_a, args.seed_a, regret_floor=args.regret_floor
    )
    b = solve_three_max_pushfold_external_dcfr(
        args.stack, args.sweeps_b, args.seed_b, regret_floor=args.regret_floor
    )
    distances = compare_strategy_sets(a.strategies, b.strategies)
    ja = 100.0 * combo_weighted_action_frequency(a.strategies["BTN_ROOT"], "jam")
    jb = 100.0 * combo_weighted_action_frequency(b.strategies["BTN_ROOT"], "jam")
    payload = {
        "schema_version": "1.0",
        "status": "SOLVER_APPROX_DIAGNOSTIC",
        "model": "THREE_MAX_EXTERNAL_STRATIFIED_DCFR_V1",
        "stack_bb": args.stack,
        "run_a": {"sweeps": args.sweeps_a, "seed": args.seed_a, "chance_samples": a.chance_samples},
        "run_b": {"sweeps": args.sweeps_b, "seed": args.seed_b, "chance_samples": b.chance_samples},
        "btn_root_jam_pct": {"a": ja, "b": jb},
        "btn_root_jam_delta_pct_points": abs(ja - jb),
        "max_combo_weighted_mae_pct": max(d.combo_weighted_mae_pct for d in distances),
        "max_hand_delta_pct": max(d.max_hand_delta_pct for d in distances),
        "distances": [d.__dict__ for d in distances],
    }
    decision = evaluate_stability_report(payload)
    payload["stability_gate"] = {"passed": decision.passed, "reasons": list(decision.reasons)}
    payload["interpretation"] = (
        "Passing means numerical stability only inside the restricted 3-player push/fold abstraction; "
        "it is not full Spin GTO certification."
    )
    out = Path(args.output)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(payload, ensure_ascii=False, indent=2, sort_keys=True), encoding="utf-8")
    print(json.dumps({
        "btn_root_jam_pct": payload["btn_root_jam_pct"],
        "btn_root_jam_delta_pct_points": payload["btn_root_jam_delta_pct_points"],
        "max_combo_weighted_mae_pct": payload["max_combo_weighted_mae_pct"],
        "stability_gate": payload["stability_gate"],
    }, indent=2))


if __name__ == "__main__":
    main()

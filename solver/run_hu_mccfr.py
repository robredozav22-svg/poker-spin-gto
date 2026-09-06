from __future__ import annotations

import argparse
import json
from pathlib import Path

from .diagnostics import estimate_hu_pushfold_best_response_gains
from .hands import HAND_CLASSES, class_combo_count
from .hu_mccfr import solve_hu_pushfold_chance_sampled
from .pushfold_hu import compatible_pair_count


def btn_aggregate(strategy: dict[str, dict[str, float]]) -> dict[str, float]:
    total = sum(class_combo_count(h) for h in HAND_CLASSES)
    jam = sum(class_combo_count(h) * strategy[h]["jam"] for h in HAND_CLASSES) / total
    return {"fold": 100.0 * (1.0 - jam), "jam": 100.0 * jam}


def bb_aggregate_conditional_on_jam(btn, bb) -> dict[str, float]:
    weights = {}
    total = 0.0
    for v in HAND_CLASSES:
        w = sum(compatible_pair_count(h, v) * btn[h]["jam"] for h in HAND_CLASSES)
        weights[v] = w
        total += w
    if total <= 0:
        return {"fold": 100.0, "call": 0.0}
    call = sum((weights[v] / total) * bb[v]["call"] for v in HAND_CLASSES)
    return {"fold": 100.0 * (1.0 - call), "call": 100.0 * call}


def main() -> None:
    parser = argparse.ArgumentParser(description="Generate our own HU push/fold chart with chance-sampled CFR")
    parser.add_argument("--stack", type=float, required=True)
    parser.add_argument("--iterations", type=int, default=1_000_000)
    parser.add_argument("--seed", type=int, default=20260906)
    parser.add_argument("--diagnostic-samples", type=int, default=50_000)
    parser.add_argument("--output", required=True)
    args = parser.parse_args()

    result = solve_hu_pushfold_chance_sampled(args.stack, args.iterations, args.seed)
    diagnostics = estimate_hu_pushfold_best_response_gains(
        result.btn,
        result.bb,
        stack_bb=args.stack,
        samples=args.diagnostic_samples,
        seed=args.seed + 1_000_000,
    )

    payload = {
        "schema_version": "1.1",
        "status": "SOLVER_APPROX",
        "production_approved": False,
        "model": "HU_CHANCE_SAMPLED_PUSH_FOLD_V1",
        "stack_bb": args.stack,
        "iterations": args.iterations,
        "seed": args.seed,
        "external_range_source": None,
        "external_equity_source": None,
        "assumptions": {
            "BTN_is_SB": True,
            "small_blind_bb": 0.5,
            "big_blind_bb": 1.0,
            "BTN_actions": ["fold", "jam"],
            "BB_facing_jam_actions": ["fold", "call"],
            "limp_allowed": False,
            "minraise_allowed": False,
            "postflop_tree": False
        },
        "aggregate_pct": {
            "BTN": btn_aggregate(result.btn),
            "BB_facing_BTN_jam": bb_aggregate_conditional_on_jam(result.btn, result.bb),
        },
        "diagnostics": {
            "samples": diagnostics.samples,
            "BTN_best_response_gain_bb": diagnostics.btn_best_response_gain_bb,
            "BB_best_response_gain_bb": diagnostics.bb_best_response_gain_bb,
            "total_unilateral_gain_bb": diagnostics.total_unilateral_gain_bb,
            "BTN_policy_ev_bb": diagnostics.btn_policy_ev_bb,
            "BB_policy_ev_bb": diagnostics.bb_policy_ev_bb,
            "zero_sum_residual_bb": diagnostics.btn_policy_ev_bb + diagnostics.bb_policy_ev_bb,
            "scope": "HU push/fold abstraction only; not full Hold'em exploitability"
        },
        "hands": {
            "BTN": result.btn,
            "BB_facing_BTN_jam": result.bb,
        }
    }

    out = Path(args.output)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(payload, ensure_ascii=False, indent=2, sort_keys=True), encoding="utf-8")
    print(json.dumps({"aggregate_pct": payload["aggregate_pct"], "diagnostics": payload["diagnostics"]}, indent=2))
    print(f"wrote {out}")


if __name__ == "__main__":
    main()

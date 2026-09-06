from __future__ import annotations

import argparse
import json
from pathlib import Path

from .convergence import combo_weighted_action_frequency
from .three_max_external_dcfr import solve_three_max_pushfold_external_dcfr


def main() -> None:
    parser = argparse.ArgumentParser(description="Generate stratified external-sampling DCFR 3-max push/fold strategy")
    parser.add_argument("--stack", type=float, required=True)
    parser.add_argument("--sweeps", type=int, default=500)
    parser.add_argument("--seed", type=int, default=20260914)
    parser.add_argument("--alpha", type=float, default=1.5)
    parser.add_argument("--beta", type=float, default=0.0)
    parser.add_argument("--gamma", type=float, default=2.0)
    parser.add_argument("--regret-floor", action="store_true")
    parser.add_argument("--output", required=True)
    args = parser.parse_args()

    result = solve_three_max_pushfold_external_dcfr(
        args.stack,
        args.sweeps,
        args.seed,
        alpha=args.alpha,
        beta=args.beta,
        gamma=args.gamma,
        regret_floor=args.regret_floor,
    )
    root_jam = 100.0 * combo_weighted_action_frequency(result.strategies["BTN_ROOT"], "jam")
    payload = {
        "schema_version": "1.0",
        "status": "SOLVER_APPROX",
        "production_approved": False,
        "model": "THREE_MAX_EXTERNAL_STRATIFIED_DCFR_V1",
        "stack_bb": result.stack_bb,
        "sweeps": result.sweeps,
        "chance_samples": result.chance_samples,
        "seed": result.seed,
        "dcfr": {
            "alpha": args.alpha,
            "beta": args.beta,
            "gamma": args.gamma,
            "regret_floor": args.regret_floor,
        },
        "external_range_source": None,
        "external_equity_source": None,
        "assumptions": {
            "blinds_bb": {"BTN": 0.0, "SB": 0.5, "BB": 1.0},
            "actions": "restricted push/fold/call tree",
            "limp_allowed": False,
            "minraise_allowed": False,
            "postflop_tree": False,
            "equal_stacks_only": True,
            "sampling": "stratified traverser hand class + external opponent actions",
        },
        "aggregate_pct": {"BTN_ROOT": {"fold": 100.0 - root_jam, "jam": root_jam}},
        "hands": result.strategies,
    }
    out = Path(args.output)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(payload, ensure_ascii=False, indent=2, sort_keys=True), encoding="utf-8")
    print(json.dumps(payload["aggregate_pct"], indent=2))


if __name__ == "__main__":
    main()

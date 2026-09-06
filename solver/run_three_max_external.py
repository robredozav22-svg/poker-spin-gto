from __future__ import annotations

import argparse
import json
from pathlib import Path

from .convergence import combo_weighted_action_frequency
from .three_max_external_mccfr import solve_three_max_pushfold_external_stratified


def main() -> None:
    parser = argparse.ArgumentParser(description="Generate stratified external-sampling 3-max push/fold strategy")
    parser.add_argument("--stack", type=float, required=True)
    parser.add_argument("--sweeps", type=int, default=500)
    parser.add_argument("--seed", type=int, default=20260912)
    parser.add_argument("--output", required=True)
    args = parser.parse_args()

    result = solve_three_max_pushfold_external_stratified(args.stack, args.sweeps, args.seed)
    root = result.strategies["BTN_ROOT"]
    root_jam = 100.0 * combo_weighted_action_frequency(root, "jam")

    payload = {
        "schema_version": "1.0",
        "status": "SOLVER_APPROX",
        "production_approved": False,
        "model": "THREE_MAX_EXTERNAL_STRATIFIED_PUSH_FOLD_V1",
        "stack_bb": result.stack_bb,
        "sweeps": result.sweeps,
        "chance_samples": result.chance_samples,
        "seed": result.seed,
        "external_range_source": None,
        "external_equity_source": None,
        "assumptions": {
            "blinds_bb": {"BTN": 0.0, "SB": 0.5, "BB": 1.0},
            "BTN_root_actions": ["fold", "jam"],
            "SB_first_in_after_BTN_fold": ["fold", "jam"],
            "responses_to_jam": ["fold", "call"],
            "limp_allowed": False,
            "minraise_allowed": False,
            "postflop_tree": False,
            "equal_stacks_only": True,
            "sampling": "stratified traverser hand class + external opponent actions"
        },
        "aggregate_pct": {
            "BTN_ROOT": {"fold": 100.0 - root_jam, "jam": root_jam}
        },
        "hands": result.strategies,
    }

    out = Path(args.output)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(payload, ensure_ascii=False, indent=2, sort_keys=True), encoding="utf-8")
    print(json.dumps(payload["aggregate_pct"], indent=2))
    print(f"wrote {out}")


if __name__ == "__main__":
    main()

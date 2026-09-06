from __future__ import annotations

import argparse
import json
from pathlib import Path

from .hands import HAND_CLASSES, class_combo_count
from .three_max_mccfr import NODE_ACTIONS, public_node_id, solve_three_max_pushfold_chance_sampled


def root_btn_aggregate(strategy: dict[str, dict[str, float]]) -> dict[str, float]:
    total = sum(class_combo_count(h) for h in HAND_CLASSES)
    jam = sum(class_combo_count(h) * strategy[h]["jam"] for h in HAND_CLASSES) / total
    return {"fold": 100.0 * (1.0 - jam), "jam": 100.0 * jam}


def main() -> None:
    parser = argparse.ArgumentParser(description="Generate our own equal-stack 3-max push/fold strategy")
    parser.add_argument("--stack", type=float, required=True)
    parser.add_argument("--iterations", type=int, default=300_000)
    parser.add_argument("--seed", type=int, default=20260906)
    parser.add_argument("--output", required=True)
    args = parser.parse_args()

    result = solve_three_max_pushfold_chance_sampled(args.stack, args.iterations, args.seed)
    root = result.strategies["BTN_ROOT"]

    node_meta = {}
    for history, (actor, actions) in NODE_ACTIONS.items():
        node_meta[public_node_id(history)] = {
            "actor": actor,
            "history": list(history),
            "actions": list(actions),
        }

    payload = {
        "schema_version": "1.0",
        "status": "SOLVER_APPROX",
        "production_approved": False,
        "model": "THREE_MAX_CHANCE_SAMPLED_PUSH_FOLD_V1",
        "stack_bb": args.stack,
        "stacks_bb": [args.stack, args.stack, args.stack],
        "iterations": args.iterations,
        "seed": args.seed,
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
            "equal_stacks_only": True
        },
        "aggregate_pct": {
            "BTN_ROOT": root_btn_aggregate(root)
        },
        "nodes": node_meta,
        "hands": result.strategies,
    }

    out = Path(args.output)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(payload, ensure_ascii=False, indent=2, sort_keys=True), encoding="utf-8")
    print(json.dumps(payload["aggregate_pct"], indent=2))
    print(f"wrote {out}")


if __name__ == "__main__":
    main()

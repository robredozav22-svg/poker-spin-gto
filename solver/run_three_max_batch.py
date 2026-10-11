from __future__ import annotations

import argparse
import json
from pathlib import Path

from .hands import HAND_CLASSES, class_combo_count
from .three_max_batch_cfr import solve_three_max_pushfold_batch_cfr


def aggregate(chart: dict[str, dict[str, float]], action: str) -> float:
    total = sum(class_combo_count(h) for h in HAND_CLASSES)
    return 100.0 * sum(class_combo_count(h) * chart[h][action] for h in HAND_CLASSES) / total


def main() -> None:
    p = argparse.ArgumentParser()
    p.add_argument("--stack", type=float, required=True)
    p.add_argument("--sweeps", type=int, default=300)
    p.add_argument("--samples-per-hand", type=int, default=8)
    p.add_argument("--corpus-seed", type=int, default=20260916)
    p.add_argument("--output", required=True)
    args = p.parse_args()

    r = solve_three_max_pushfold_batch_cfr(
        stack_bb=args.stack,
        sweeps=args.sweeps,
        corpus_seed=args.corpus_seed,
        samples_per_hand=args.samples_per_hand,
    )
    root = r.strategies["BTN_ROOT"]
    payload = {
        "schema_version": "1.0",
        "status": "SOLVER_APPROX_FIXED_CORPUS",
        "production_approved": False,
        "model": "THREE_MAX_FIXED_CORPUS_BATCH_CFR_V1",
        "stack_bb": r.stack_bb,
        "sweeps": r.sweeps,
        "corpus_seed": r.corpus_seed,
        "samples_per_hand": r.samples_per_hand,
        "chance_samples": r.chance_samples,
        "external_range_source": None,
        "external_equity_source": None,
        "aggregate_pct": {
            "BTN_ROOT": {
                "fold": aggregate(root, "fold"),
                "jam": aggregate(root, "jam"),
            }
        },
        "assumptions": {
            "tree": "restricted 3max push/fold/call",
            "chance": "blocker-aware frozen conditioned corpus",
            "opponent_actions": "fully enumerated, not sampled",
            "postflop_tree": False,
            "limp_allowed": False,
            "minraise_allowed": False,
            "equal_stacks_only": True,
        },
        "hands": r.strategies,
    }
    out = Path(args.output)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(payload, indent=2, sort_keys=True), encoding="utf-8")
    print(json.dumps(payload["aggregate_pct"], indent=2))


if __name__ == "__main__":
    main()

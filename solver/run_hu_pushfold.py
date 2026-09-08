from __future__ import annotations

import argparse
import json
from pathlib import Path

from .hands import HAND_CLASSES, class_combo_count
from .pushfold_hu import compatible_pair_count, solve_hu_pushfold, validate_equity_table


def btn_aggregate(strategy: dict[str, dict[str, float]]) -> dict[str, float]:
    total = sum(class_combo_count(h) for h in HAND_CLASSES)
    jam = sum(class_combo_count(h) * strategy[h]["jam"] for h in HAND_CLASSES) / total
    return {"fold": 100.0 * (1.0 - jam), "jam": 100.0 * jam}


def bb_aggregate_conditional_on_jam(
    btn_strategy: dict[str, dict[str, float]],
    bb_strategy: dict[str, dict[str, float]],
) -> dict[str, float]:
    weights: dict[str, float] = {}
    total = 0.0
    for v in HAND_CLASSES:
        w = 0.0
        for h in HAND_CLASSES:
            w += compatible_pair_count(h, v) * btn_strategy[h]["jam"]
        weights[v] = w
        total += w

    if total <= 0:
        return {"fold": 100.0, "call": 0.0}

    call = sum((weights[v] / total) * bb_strategy[v]["call"] for v in HAND_CLASSES)
    return {"fold": 100.0 * (1.0 - call), "call": 100.0 * call}


def main() -> None:
    parser = argparse.ArgumentParser(description="Solve our own HU push/fold chart from a self-generated equity table")
    parser.add_argument("equity_table")
    parser.add_argument("--stack", type=float, required=True)
    parser.add_argument("--iterations", type=int, default=20_000)
    parser.add_argument("--output", required=True)
    args = parser.parse_args()

    source = json.loads(Path(args.equity_table).read_text(encoding="utf-8"))
    equity = source.get("equity")
    if not isinstance(equity, dict):
        raise SystemExit("Invalid equity table: missing equity object")
    validate_equity_table(equity)

    solution = solve_hu_pushfold(equity, stack_bb=args.stack, iterations=args.iterations)
    source_status = source.get("meta", {}).get("status", "UNKNOWN")

    payload = {
        "schema_version": "1.0",
        "status": "SOLVER_APPROX",
        "production_approved": False,
        "model": "HU_PUSH_FOLD_ABSTRACTION_V1",
        "stack_bb": args.stack,
        "iterations": args.iterations,
        "equity_source_status": source_status,
        "assumptions": {
            "positions": ["BTN_SB", "BB"],
            "small_blind_bb": 0.5,
            "big_blind_bb": 1.0,
            "btn_actions": ["fold", "jam"],
            "bb_facing_jam_actions": ["fold", "call"],
            "limp_allowed": False,
            "minraise_allowed": False,
            "postflop_tree": False
        },
        "aggregate_pct": {
            "BTN": btn_aggregate(solution["BTN"]),
            "BB_facing_BTN_jam": bb_aggregate_conditional_on_jam(solution["BTN"], solution["BB"]),
        },
        "hands": {
            "BTN": solution["BTN"],
            "BB_facing_BTN_jam": solution["BB"],
        },
    }

    out = Path(args.output)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(payload, ensure_ascii=False, indent=2, sort_keys=True), encoding="utf-8")
    print(json.dumps(payload["aggregate_pct"], indent=2))
    print(f"wrote {out}")


if __name__ == "__main__":
    main()

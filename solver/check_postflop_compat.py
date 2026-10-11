from __future__ import annotations

import argparse
import json

from .postflop_solution_reader import read_solution_meta


def main() -> None:
    p = argparse.ArgumentParser()
    p.add_argument("--solution", required=True)
    args = p.parse_args()
    meta = read_solution_meta(args.solution)
    payload = {
        "engine_version": meta.engine_version,
        "iterations": meta.iterations,
        "exploitability_pct_of_pot": meta.exploitability_pct_of_pot,
        "payoff_unit": meta.payoff_unit,
        "assumptions_hash": meta.assumptions_hash,
        "has_root_combo_labels": meta.has_root_combo_labels,
        "has_node_strategies": meta.has_node_strategies,
        "has_per_hand_evs": meta.has_per_hand_evs,
        "continuation_ready": bool(
            meta.has_root_combo_labels and meta.has_node_strategies and meta.has_per_hand_evs
        ),
    }
    if not meta.has_root_combo_labels or not meta.has_node_strategies:
        raise SystemExit("measured postflop solution lacks auditable root combos/strategies")
    print(json.dumps(payload, indent=2, sort_keys=True))


if __name__ == "__main__":
    main()

from __future__ import annotations

import json
from pathlib import Path


def load_btn_rfi_anchor(path: str | Path, stack_bb: float) -> dict[str, float]:
    data = json.loads(Path(path).read_text(encoding="utf-8"))
    for row in data["rfi_3max"]:
        if row["position"] == "BTN" and float(row["stack_bb"]) == float(stack_bb):
            total = float(data["total_starting_combos"])
            return {k.lower(): 100.0 * float(v) / total for k, v in row["action_combos"].items()}
    raise KeyError(f"no BTN RFI anchor for {stack_bb}bb")


def compare_btn_root_to_anchor(
    solver_payload: dict,
    anchor_path: str | Path,
) -> dict:
    stack = float(solver_payload["stack_bb"])
    anchor = load_btn_rfi_anchor(anchor_path, stack)
    solver = solver_payload["aggregate_pct"]["BTN_ROOT"]
    common = sorted(set(anchor) & set(solver))
    return {
        "stack_bb": stack,
        "solver_status": solver_payload.get("status"),
        "anchor_role": "REFERENCE_CROSSCHECK_NOT_TRAINING_LABEL",
        "solver_pct": solver,
        "reference_pct": anchor,
        "common_action_delta_pct_points": {
            a: float(solver[a]) - float(anchor[a]) for a in common
        },
        "note": (
            "A small delta is useful validation evidence; a large delta may mean "
            "non-convergence OR a different/missing game-tree assumption. This "
            "comparison must never tune hand frequencies directly to the reference."
        ),
    }

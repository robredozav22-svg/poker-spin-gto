from __future__ import annotations

from dataclasses import dataclass
import hashlib
import json
from pathlib import Path


class PostflopSolutionFormatError(ValueError):
    pass


@dataclass(frozen=True)
class ExternalSolutionMeta:
    engine_version: str
    iterations: int
    exploitability_pct_of_pot: float
    exploitability_chips: float
    payoff_unit: str
    assumptions_hash: str
    has_root_combo_labels: bool
    has_node_strategies: bool
    has_per_hand_evs: bool


def _canonical_hash(config: object) -> str:
    raw = json.dumps(config, sort_keys=True, separators=(",", ":"), ensure_ascii=False)
    return hashlib.sha256(raw.encode("utf-8")).hexdigest()


def read_solution_meta(path: str | Path) -> ExternalSolutionMeta:
    """Read only the auditable metadata needed before a solution may be consumed.

    The currently inspected MIT `ucsandman/postflop` solution format persists
    config, measured exploitability, root EVs, root combo labels, and average
    strategies per decision node. It does NOT persist per-combo EVs. Our
    preflop continuation bridge therefore treats such a file as a valid solved
    strategy artifact but an *insufficient continuation-value artifact* until a
    per-combo/per-hand EV export is added.
    """
    data = json.loads(Path(path).read_text(encoding="utf-8"))
    if not isinstance(data, dict):
        raise PostflopSolutionFormatError("solution root must be an object")

    config = data.get("config")
    meta = data.get("meta")
    nodes = data.get("nodes")
    root_combos = data.get("root_combos")
    if not isinstance(config, dict):
        raise PostflopSolutionFormatError("missing config")
    if not isinstance(meta, dict):
        raise PostflopSolutionFormatError("missing meta")
    if not isinstance(nodes, list):
        raise PostflopSolutionFormatError("missing nodes")
    if not isinstance(root_combos, list) or len(root_combos) != 2:
        raise PostflopSolutionFormatError("missing two root combo lists")

    try:
        engine_version = str(meta["engine_version"])
        iterations = int(meta["iterations"])
        exploit_pct = float(meta["exploitability_pct_of_pot"])
        exploit_chips = float(meta["exploitability_chips"])
    except (KeyError, TypeError, ValueError) as exc:
        raise PostflopSolutionFormatError(f"invalid measured solve metadata: {exc}") from exc

    if iterations <= 0:
        raise PostflopSolutionFormatError("iterations must be positive")
    if exploit_pct < 0 or exploit_chips < 0:
        raise PostflopSolutionFormatError("exploitability cannot be negative")

    payoff_unit = str(meta.get("payoff_unit", "chips"))
    has_node_strategies = all(
        isinstance(n, dict)
        and isinstance(n.get("strategy"), list)
        and isinstance(n.get("actions"), list)
        and isinstance(n.get("combo_count"), int)
        for n in nodes
    )
    has_root_combo_labels = all(isinstance(side, list) and side for side in root_combos)

    # No field in the inspected format stores per-combo EV arrays. Keep the
    # detection generic so a future backend extension becomes usable without
    # weakening this guard.
    has_per_hand_evs = any(
        isinstance(n, dict) and any(k in n for k in ("evs", "combo_evs", "hand_evs"))
        for n in nodes
    ) or any(k in data for k in ("combo_evs", "hand_evs"))

    return ExternalSolutionMeta(
        engine_version=engine_version,
        iterations=iterations,
        exploitability_pct_of_pot=exploit_pct,
        exploitability_chips=exploit_chips,
        payoff_unit=payoff_unit,
        assumptions_hash=_canonical_hash(config),
        has_root_combo_labels=has_root_combo_labels,
        has_node_strategies=has_node_strategies,
        has_per_hand_evs=has_per_hand_evs,
    )


def require_continuation_capable(meta: ExternalSolutionMeta, max_exploitability_pct: float) -> None:
    if meta.exploitability_pct_of_pot > max_exploitability_pct:
        raise PostflopSolutionFormatError(
            f"postflop solve exploitability {meta.exploitability_pct_of_pot:.6f}% exceeds "
            f"limit {max_exploitability_pct:.6f}%"
        )
    if not meta.has_node_strategies or not meta.has_root_combo_labels:
        raise PostflopSolutionFormatError("solution lacks strategy/combo labeling required for audit")
    if not meta.has_per_hand_evs:
        raise PostflopSolutionFormatError(
            "solution is measured but does not export per-combo/per-hand EVs; "
            "cannot be used as a preflop continuation-value source"
        )

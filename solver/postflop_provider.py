from __future__ import annotations

from dataclasses import dataclass
from typing import Protocol


@dataclass(frozen=True)
class PostflopSolveRequest:
    node_id: str
    board: str
    oop_range: str
    ip_range: str
    effective_stack_bb: float
    starting_pot_bb: float
    oop_position: str
    ip_position: str
    sizing_profile: str
    payout_profile: str = "wta"


@dataclass(frozen=True)
class HandContinuationEV:
    hand_class: str
    ev_bb: float


@dataclass(frozen=True)
class PostflopSolveResult:
    node_id: str
    provider: str
    provider_version: str
    exploitability_pct_pot: float | None
    status: str
    hand_evs: tuple[HandContinuationEV, ...]
    assumptions_hash: str


class PostflopSolverProvider(Protocol):
    """Backend contract for locally computed postflop continuation values.

    Providers calculate a solution from game inputs. They must not return a
    downloaded chart/range library as if it were a solve.
    """

    name: str

    def solve(self, request: PostflopSolveRequest) -> PostflopSolveResult:
        ...


def validate_result(result: PostflopSolveResult) -> None:
    if result.status not in {"POSTFLOP_SOLVED_APPROX", "POSTFLOP_SOLVED_MEASURED"}:
        raise ValueError(f"unsupported postflop status: {result.status}")
    if not result.provider or not result.provider_version:
        raise ValueError("postflop result requires provider provenance")
    if not result.assumptions_hash:
        raise ValueError("postflop result requires assumptions hash")
    seen = set()
    for item in result.hand_evs:
        if item.hand_class in seen:
            raise ValueError(f"duplicate hand EV: {item.hand_class}")
        seen.add(item.hand_class)
    if result.status == "POSTFLOP_SOLVED_MEASURED":
        if result.exploitability_pct_pot is None:
            raise ValueError("measured solve requires exploitability")
        if result.exploitability_pct_pot < 0:
            raise ValueError("exploitability cannot be negative")

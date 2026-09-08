from __future__ import annotations

from dataclasses import dataclass

from .model import Action, DecisionNodeKey, Mode, PayoutProfile, TreeAction


@dataclass(frozen=True)
class ContinuationRequest:
    id: str
    node: DecisionNodeKey
    active_players: tuple[str, ...]
    pot_bb: float
    stacks_behind_bb: tuple[float, ...]
    spr: float
    source_status: str


def _req(
    request_id: str,
    history: tuple[TreeAction, ...],
    hero: str,
    active_players: tuple[str, ...],
    pot_bb: float,
    stacks_behind_bb: tuple[float, ...],
    source_status: str = "SCREEN_TREE_CROSSCHECK",
) -> ContinuationRequest:
    if len(active_players) != len(stacks_behind_bb):
        raise ValueError("active_players/stacks_behind length mismatch")
    if pot_bb <= 0:
        raise ValueError("pot must be positive")
    min_stack = min(stacks_behind_bb)
    spr = min_stack / pot_bb
    node = DecisionNodeKey(
        mode=Mode.THREE_MAX,
        payout_profile=PayoutProfile.WTA,
        stacks_bb=(15.0, 15.0, 15.0),
        hero=hero,
        history=history,
    )
    return ContinuationRequest(
        id=request_id,
        node=node,
        active_players=active_players,
        pot_bb=pot_bb,
        stacks_behind_bb=stacks_behind_bb,
        spr=spr,
        source_status=source_status,
    )


def fifteen_bb_core_postflop_requests() -> tuple[ContinuationRequest, ...]:
    """Core flop subgames required by the recovered 15bb Spin action tree.

    These are state definitions only; no EV is guessed or supplied here.
    """
    return (
        _req(
            "15_BTN_R2_SB_FOLD_BB_CALL",
            (
                TreeAction("BTN", Action.RAISE, 2.0),
                TreeAction("SB", Action.FOLD),
                TreeAction("BB", Action.CALL),
            ),
            hero="BTN",
            active_players=("BTN", "BB"),
            pot_bb=4.5,  # BTN 2 + dead SB 0.5 + BB 2
            stacks_behind_bb=(13.0, 13.0),
        ),
        _req(
            "15_BTN_R2_SB_CALL_BB_CALL",
            (
                TreeAction("BTN", Action.RAISE, 2.0),
                TreeAction("SB", Action.CALL),
                TreeAction("BB", Action.CALL),
            ),
            hero="BTN",
            active_players=("BTN", "SB", "BB"),
            pot_bb=6.0,
            stacks_behind_bb=(13.0, 13.0, 13.0),
        ),
        _req(
            "15_BTN_FOLD_SB_LIMP_BB_CHECK",
            (
                TreeAction("BTN", Action.FOLD),
                TreeAction("SB", Action.LIMP, 1.0),
                TreeAction("BB", Action.CHECK),
            ),
            hero="SB",
            active_players=("SB", "BB"),
            pot_bb=2.0,
            stacks_behind_bb=(14.0, 14.0),
        ),
        _req(
            "15_BTN_FOLD_SB_R3_BB_CALL",
            (
                TreeAction("BTN", Action.FOLD),
                TreeAction("SB", Action.RAISE, 3.0),
                TreeAction("BB", Action.CALL),
            ),
            hero="SB",
            active_players=("SB", "BB"),
            pot_bb=6.0,
            stacks_behind_bb=(12.0, 12.0),
        ),
    )

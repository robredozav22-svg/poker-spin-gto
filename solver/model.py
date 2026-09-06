from __future__ import annotations

from dataclasses import dataclass
from enum import Enum


class Mode(str, Enum):
    THREE_MAX = "3max"
    HU = "hu"


class PayoutProfile(str, Enum):
    WTA = "wta"
    ICM = "icm"


class Action(str, Enum):
    FOLD = "fold"
    CHECK = "check"
    CALL = "call"
    LIMP = "limp"
    RAISE = "raise"
    JAM = "jam"


@dataclass(frozen=True)
class GameConfig:
    mode: Mode
    payout_profile: PayoutProfile
    stacks_bb: tuple[float, ...]
    small_blind_bb: float = 0.5
    big_blind_bb: float = 1.0
    ante_bb: float = 0.0

    def __post_init__(self) -> None:
        expected = 3 if self.mode == Mode.THREE_MAX else 2
        if len(self.stacks_bb) != expected:
            raise ValueError(f"{self.mode.value} requires {expected} stacks")
        if any(s <= 0 for s in self.stacks_bb):
            raise ValueError("All stacks must be positive")
        if self.small_blind_bb <= 0 or self.big_blind_bb <= 0:
            raise ValueError("Blinds must be positive")
        if self.small_blind_bb >= self.big_blind_bb:
            raise ValueError("Small blind must be smaller than big blind")

    @property
    def effective_stack_bb(self) -> float:
        return min(self.stacks_bb)


@dataclass(frozen=True)
class TreeAction:
    actor: str
    action: Action
    size_bb: float | None = None

    def token(self) -> str:
        if self.size_bb is None:
            return f"{self.actor}_{self.action.value.upper()}"
        size = int(self.size_bb) if float(self.size_bb).is_integer() else self.size_bb
        return f"{self.actor}_{self.action.value.upper()}_{size}"


@dataclass(frozen=True)
class DecisionNodeKey:
    mode: Mode
    payout_profile: PayoutProfile
    stacks_bb: tuple[float, ...]
    hero: str
    history: tuple[TreeAction, ...]

    def canonical(self) -> str:
        stacks = "-".join(str(int(s)) if float(s).is_integer() else str(s) for s in self.stacks_bb)
        history = ">".join(a.token() for a in self.history) or "ROOT"
        return f"{self.mode.value}|{self.payout_profile.value}|{stacks}|{history}|{self.hero}"

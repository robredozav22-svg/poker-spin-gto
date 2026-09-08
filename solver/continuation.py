from __future__ import annotations

from dataclasses import dataclass

from .model import DecisionNodeKey


class MissingContinuationValue(KeyError):
    """Raised when a non-all-in branch has no approved continuation EV."""


@dataclass(frozen=True)
class ContinuationStateKey:
    node: DecisionNodeKey
    hand_class: str
    pot_bb: float
    spr: float

    def canonical(self) -> str:
        return (
            f"{self.node.canonical()}|HAND={self.hand_class}|"
            f"POT={self.pot_bb:.4f}|SPR={self.spr:.4f}"
        )


@dataclass(frozen=True)
class ContinuationValue:
    ev_bb: float
    source: str
    status: str
    samples: int | None = None
    abstraction: str | None = None


class ContinuationValueTable:
    """Strict store for postflop continuation values.

    The preflop solver must call `require()` when an action reaches a
    non-all-in postflop state. Missing values are fatal by design: we never
    replace them with raw equity, hand strength, or an interpolated guess.
    """

    def __init__(self) -> None:
        self._values: dict[str, ContinuationValue] = {}

    def put(self, key: ContinuationStateKey, value: ContinuationValue) -> None:
        if value.status not in {"CONTINUATION_EXACT", "CONTINUATION_APPROX"}:
            raise ValueError(f"unsupported continuation status: {value.status}")
        if not value.source:
            raise ValueError("continuation value requires provenance")
        self._values[key.canonical()] = value

    def get(self, key: ContinuationStateKey) -> ContinuationValue | None:
        return self._values.get(key.canonical())

    def require(self, key: ContinuationStateKey) -> ContinuationValue:
        value = self.get(key)
        if value is None:
            raise MissingContinuationValue(
                "No approved postflop continuation value for " + key.canonical()
            )
        return value

    def __len__(self) -> int:
        return len(self._values)

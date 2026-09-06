from __future__ import annotations

from dataclasses import dataclass
from enum import Enum


class PositionRole(str, Enum):
    IP = "ip"
    OOP = "oop"
    MULTIWAY = "multiway"


class Initiative(str, Enum):
    AGGRESSOR = "aggressor"
    DEFENDER = "defender"
    NONE = "none"


class MissingRealizationCoefficient(KeyError):
    pass


@dataclass(frozen=True)
class RealizationContext:
    role: PositionRole
    initiative: Initiative
    spr_band: str
    players: int

    def __post_init__(self) -> None:
        if self.players not in (2, 3):
            raise ValueError("players must be 2 or 3 for current Spin model")

    def canonical(self) -> str:
        return f"{self.players}p|{self.role.value}|{self.initiative.value}|{self.spr_band}"


@dataclass(frozen=True)
class RealizationObservation:
    context: RealizationContext
    hand_class: str
    raw_equity: float
    realized_pot_share: float
    weight: float = 1.0
    source: str = "SELF_POSTFLOP_SOLVE"

    def __post_init__(self) -> None:
        if not (0.0 < self.raw_equity <= 1.0):
            raise ValueError("raw_equity must be in (0,1]")
        if not (0.0 <= self.realized_pot_share <= 1.0):
            raise ValueError("realized_pot_share must be in [0,1]")
        if self.weight <= 0:
            raise ValueError("weight must be positive")
        if not self.source:
            raise ValueError("source is required")

    @property
    def coefficient(self) -> float:
        return self.realized_pot_share / self.raw_equity


@dataclass(frozen=True)
class RealizationCoefficient:
    value: float
    observations: int
    total_weight: float
    source: str
    status: str = "REALIZATION_APPROX"


class RealizationModel:
    """Strict realization-factor store trained only from our own postflop data.

    This model never falls back to R=1. Missing coefficients are fatal for
    production preflop solving because that fallback was shown to distort
    defend/call widths dramatically in other solver research.
    """

    def __init__(self) -> None:
        self._values: dict[tuple[str, str], RealizationCoefficient] = {}

    def put(self, context: RealizationContext, hand_class: str, value: RealizationCoefficient) -> None:
        if value.value <= 0:
            raise ValueError("realization coefficient must be positive")
        if not value.source:
            raise ValueError("realization coefficient needs provenance")
        self._values[(context.canonical(), hand_class)] = value

    def get(self, context: RealizationContext, hand_class: str) -> RealizationCoefficient | None:
        return self._values.get((context.canonical(), hand_class))

    def require(self, context: RealizationContext, hand_class: str) -> RealizationCoefficient:
        value = self.get(context, hand_class)
        if value is None:
            raise MissingRealizationCoefficient(f"missing R for {context.canonical()} {hand_class}")
        return value


def fit_realization_observations(observations: list[RealizationObservation]) -> RealizationModel:
    """Fit simple weighted means per context/hand without external labels.

    More advanced smoothing may be added later, but this first fitter avoids
    any implicit cross-hand interpolation: every coefficient must be supported
    by observations for that exact hand class and context.
    """
    grouped: dict[tuple[RealizationContext, str], list[RealizationObservation]] = {}
    for obs in observations:
        grouped.setdefault((obs.context, obs.hand_class), []).append(obs)

    model = RealizationModel()
    for (context, hand), rows in grouped.items():
        total_weight = sum(r.weight for r in rows)
        value = sum(r.weight * r.coefficient for r in rows) / total_weight
        model.put(
            context,
            hand,
            RealizationCoefficient(
                value=value,
                observations=len(rows),
                total_weight=total_weight,
                source="SELF_POSTFLOP_OBSERVATIONS",
            ),
        )
    return model


def normalized_realized_shares(
    equities: dict[str, float],
    coefficients: dict[str, float],
) -> dict[str, float]:
    """Convert raw equity × realization weights into pot-conserving shares.

    This is a leaf-value model, not a substitute for postflop solving. It is
    permitted only when coefficients came from approved self-generated solves.
    """
    if set(equities) != set(coefficients):
        raise ValueError("equities/coefficients player sets must match")
    weighted: dict[str, float] = {}
    for player, equity in equities.items():
        r = coefficients[player]
        if equity < 0 or equity > 1:
            raise ValueError("equity must be in [0,1]")
        if r <= 0:
            raise ValueError("realization coefficient must be positive")
        weighted[player] = equity * r
    total = sum(weighted.values())
    if total <= 0:
        raise ValueError("weighted equity total must be positive")
    return {p: v / total for p, v in weighted.items()}


def spr_band(spr: float) -> str:
    if spr < 0:
        raise ValueError("SPR cannot be negative")
    if spr <= 2.5:
        return "LE2_5"
    if spr <= 4.0:
        return "LE4"
    if spr <= 8.0:
        return "LE8"
    if spr <= 15.0:
        return "LE15"
    return "GT15"

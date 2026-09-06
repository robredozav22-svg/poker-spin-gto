from __future__ import annotations

from dataclasses import dataclass
from itertools import combinations
from random import Random
from typing import Iterable

from .evaluator import compare_seven
from .hands import expand_hand_class, full_deck, validate_combo


@dataclass(frozen=True)
class EquityResult:
    wins: int
    ties: int
    losses: int

    @property
    def total(self) -> int:
        return self.wins + self.ties + self.losses

    @property
    def equity(self) -> float:
        if self.total == 0:
            raise ZeroDivisionError("No boards evaluated")
        return (self.wins + 0.5 * self.ties) / self.total


def _score_board(hero: tuple[str, str], villain: tuple[str, str], board: Iterable[str]) -> int:
    board = tuple(board)
    return compare_seven(hero + board, villain + board)


def concrete_equity_exact(hero: tuple[str, str], villain: tuple[str, str]) -> EquityResult:
    """Exhaustively enumerate every legal five-card board.

    This is mathematically exact for two concrete hole-card combinations, but
    intentionally slow (~1.7M boards per matchup). It is a correctness oracle,
    not the production bulk-table builder.
    """
    validate_combo(hero)
    validate_combo(villain)
    blocked = set(hero + villain)
    if len(blocked) != 4:
        raise ValueError("Hero and villain hole cards overlap")
    deck = [c for c in full_deck() if c not in blocked]

    w = t = l = 0
    for board in combinations(deck, 5):
        cmp = _score_board(hero, villain, board)
        if cmp > 0:
            w += 1
        elif cmp < 0:
            l += 1
        else:
            t += 1
    return EquityResult(w, t, l)


def concrete_equity_sampled(
    hero: tuple[str, str],
    villain: tuple[str, str],
    boards: int = 20_000,
    seed: int = 1,
) -> EquityResult:
    """Deterministic Monte-Carlo equity for development/smoke testing.

    Never label sampled output VERIFIED_EXACT.
    """
    if boards <= 0:
        raise ValueError("boards must be positive")
    validate_combo(hero)
    validate_combo(villain)
    blocked = set(hero + villain)
    if len(blocked) != 4:
        raise ValueError("Hero and villain hole cards overlap")
    deck = [c for c in full_deck() if c not in blocked]
    rng = Random(seed)

    w = t = l = 0
    for _ in range(boards):
        board = rng.sample(deck, 5)
        cmp = _score_board(hero, villain, board)
        if cmp > 0:
            w += 1
        elif cmp < 0:
            l += 1
        else:
            t += 1
    return EquityResult(w, t, l)


def class_vs_class_sampled(
    hero_class: str,
    villain_class: str,
    boards_per_combo_pair: int = 500,
    seed: int = 1,
) -> float:
    """Average blocker-aware equity across compatible concrete combos.

    This is a self-generated approximation used to bootstrap and test the
    pipeline. It is not an approved production solution source.
    """
    hero_combos = expand_hand_class(hero_class)
    villain_combos = expand_hand_class(villain_class)
    weighted_equity = 0.0
    matchups = 0
    local_seed = seed

    for h in hero_combos:
        for v in villain_combos:
            if set(h) & set(v):
                continue
            result = concrete_equity_sampled(h, v, boards_per_combo_pair, local_seed)
            local_seed += 1
            weighted_equity += result.equity
            matchups += 1

    if matchups == 0:
        raise ValueError("No compatible concrete combo matchups")
    return weighted_equity / matchups

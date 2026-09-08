from __future__ import annotations

from collections import Counter
from itertools import combinations

RANK_VALUE = {r: i for i, r in enumerate("23456789TJQKA", start=2)}


def _straight_high(values: list[int]) -> int | None:
    uniq = sorted(set(values), reverse=True)
    if 14 in uniq:
        uniq.append(1)
    for i in range(len(uniq) - 4):
        window = uniq[i : i + 5]
        if window[0] - window[4] == 4:
            return window[0]
    return None


def evaluate_five(cards: tuple[str, ...] | list[str]) -> tuple[int, ...]:
    """Return a lexicographically comparable five-card rank tuple.

    Categories: 8 straight-flush, 7 quads, 6 full-house, 5 flush,
    4 straight, 3 trips, 2 two-pair, 1 pair, 0 high-card.
    """
    if len(cards) != 5 or len(set(cards)) != 5:
        raise ValueError("evaluate_five requires five unique cards")

    values = [RANK_VALUE[c[0]] for c in cards]
    suits = [c[1] for c in cards]
    counts = Counter(values)
    groups = sorted(((n, v) for v, n in counts.items()), reverse=True)
    is_flush = len(set(suits)) == 1
    straight_high = _straight_high(values)

    if is_flush and straight_high is not None:
        return (8, straight_high)

    quads = sorted((v for v, n in counts.items() if n == 4), reverse=True)
    if quads:
        q = quads[0]
        kicker = max(v for v in values if v != q)
        return (7, q, kicker)

    trips = sorted((v for v, n in counts.items() if n == 3), reverse=True)
    pairs = sorted((v for v, n in counts.items() if n == 2), reverse=True)
    if trips and (len(trips) > 1 or pairs):
        top_trip = trips[0]
        pair_value = trips[1] if len(trips) > 1 else pairs[0]
        return (6, top_trip, pair_value)

    if is_flush:
        return (5, *sorted(values, reverse=True))

    if straight_high is not None:
        return (4, straight_high)

    if trips:
        t = trips[0]
        kickers = sorted((v for v in values if v != t), reverse=True)
        return (3, t, *kickers)

    if len(pairs) >= 2:
        hi, lo = pairs[:2]
        kicker = max(v for v in values if v not in (hi, lo))
        return (2, hi, lo, kicker)

    if len(pairs) == 1:
        p = pairs[0]
        kickers = sorted((v for v in values if v != p), reverse=True)
        return (1, p, *kickers)

    return (0, *sorted(values, reverse=True))


def evaluate_seven(cards: tuple[str, ...] | list[str]) -> tuple[int, ...]:
    if len(cards) != 7 or len(set(cards)) != 7:
        raise ValueError("evaluate_seven requires seven unique cards")
    return max(evaluate_five(c) for c in combinations(cards, 5))


def compare_seven(hero: tuple[str, ...] | list[str], villain: tuple[str, ...] | list[str]) -> int:
    """Return 1 if hero wins, -1 if villain wins, 0 on tie."""
    a = evaluate_seven(hero)
    b = evaluate_seven(villain)
    return (a > b) - (a < b)

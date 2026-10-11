from __future__ import annotations

from itertools import combinations

RANKS = "AKQJT98765432"
SUITS = "shdc"
RANK_INDEX = {r: i for i, r in enumerate(RANKS)}


def all_hand_classes() -> list[str]:
    """Return the canonical 169 Hold'em starting-hand classes."""
    out: list[str] = []
    for i, r1 in enumerate(RANKS):
        for j, r2 in enumerate(RANKS):
            if i == j:
                out.append(r1 + r2)
            elif i < j:
                out.append(r1 + r2 + "s")
            else:
                out.append(r2 + r1 + "o")
    return list(dict.fromkeys(out))


def normalize_class(hand: str) -> str:
    hand = hand.strip()
    if len(hand) == 2 and hand[0] == hand[1] and hand[0] in RANKS:
        return hand
    if len(hand) != 3 or hand[2] not in "so":
        raise ValueError(f"Invalid hand class: {hand}")
    r1, r2, suitedness = hand[0], hand[1], hand[2]
    if r1 not in RANKS or r2 not in RANKS or r1 == r2:
        raise ValueError(f"Invalid hand class: {hand}")
    if RANK_INDEX[r1] > RANK_INDEX[r2]:
        r1, r2 = r2, r1
    return r1 + r2 + suitedness


def class_combo_count(hand: str) -> int:
    hand = normalize_class(hand)
    if len(hand) == 2:
        return 6
    return 4 if hand.endswith("s") else 12


def expand_hand_class(hand: str) -> list[tuple[str, str]]:
    """Expand a hand class into exact two-card combinations."""
    hand = normalize_class(hand)
    if len(hand) == 2:
        rank = hand[0]
        return [(rank + a, rank + b) for a, b in combinations(SUITS, 2)]

    r1, r2, suitedness = hand
    if suitedness == "s":
        return [(r1 + s, r2 + s) for s in SUITS]

    out: list[tuple[str, str]] = []
    for s1 in SUITS:
        for s2 in SUITS:
            if s1 != s2:
                out.append((r1 + s1, r2 + s2))
    return out


def combo_to_class(combo: tuple[str, str]) -> str:
    validate_combo(combo)
    a, b = combo
    r1, s1 = a[0], a[1]
    r2, s2 = b[0], b[1]
    if r1 == r2:
        return r1 + r2
    if RANK_INDEX[r1] > RANK_INDEX[r2]:
        r1, r2 = r2, r1
        s1, s2 = s2, s1
    return r1 + r2 + ("s" if s1 == s2 else "o")


def full_deck() -> list[str]:
    return [r + s for r in RANKS for s in SUITS]


def validate_combo(combo: tuple[str, str]) -> None:
    if len(combo) != 2 or combo[0] == combo[1]:
        raise ValueError(f"Invalid combo: {combo}")
    deck = set(full_deck())
    if combo[0] not in deck or combo[1] not in deck:
        raise ValueError(f"Invalid combo: {combo}")


HAND_CLASSES = all_hand_classes()
assert len(HAND_CLASSES) == 169
assert sum(class_combo_count(h) for h in HAND_CLASSES) == 1326

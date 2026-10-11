from __future__ import annotations

from typing import Mapping, Sequence


Player = str


def settle_pots(
    contributions: Mapping[Player, float],
    folded: Mapping[Player, bool],
    hand_ranks: Mapping[Player, tuple[int, ...]],
) -> dict[Player, float]:
    """Settle arbitrary main/side pots and return net chip payoff per player.

    `contributions` includes blinds and every later chip committed this hand.
    Folded players' chips remain in pots but they are never eligible to win.
    `hand_ranks` contains comparable showdown ranks for all non-folded players.

    The result is net relative to the beginning of the hand: chips received
    minus chips contributed. Therefore payoffs sum to zero (modulo float noise).
    """
    players = list(contributions)
    if not players:
        raise ValueError("No players")
    if set(folded) != set(players):
        raise ValueError("folded keys must match contributions")
    if any(v < 0 for v in contributions.values()):
        raise ValueError("Negative contribution")

    live = [p for p in players if not folded[p]]
    if not live:
        raise ValueError("At least one player must remain live")
    missing_ranks = [p for p in live if p not in hand_ranks]
    if len(live) > 1 and missing_ranks:
        raise ValueError(f"Missing showdown ranks: {missing_ranks}")

    received = {p: 0.0 for p in players}
    levels = sorted({float(v) for v in contributions.values() if v > 0})
    previous = 0.0

    for level in levels:
        contributors = [p for p in players if contributions[p] >= level]
        pot = (level - previous) * len(contributors)
        previous = level
        if pot <= 0:
            continue

        eligible = [p for p in contributors if not folded[p]]
        if not eligible:
            raise ValueError("Pot tier has no eligible player")

        if len(eligible) == 1:
            winners = eligible
        else:
            best = max(hand_ranks[p] for p in eligible)
            winners = [p for p in eligible if hand_ranks[p] == best]

        share = pot / len(winners)
        for p in winners:
            received[p] += share

    payoff = {p: received[p] - float(contributions[p]) for p in players}
    residual = sum(payoff.values())
    if abs(residual) > 1e-9:
        raise AssertionError(f"Pot settlement is not zero-sum: {residual}")
    return payoff


def contributions_after_all_in(
    stacks_bb: Mapping[Player, float],
    blinds_bb: Mapping[Player, float],
    all_in_players: Sequence[Player],
    folded_players: Sequence[Player] = (),
) -> dict[Player, float]:
    """Build total contributions for a terminal all-in state.

    Players listed as all-in commit their full starting stack. Folded players
    retain only whatever blind amount is provided here; more complex histories
    should pass their exact contributions directly to `settle_pots`.
    """
    if set(stacks_bb) != set(blinds_bb):
        raise ValueError("stack/blind keys must match")
    all_in = set(all_in_players)
    folded = set(folded_players)
    if all_in & folded:
        raise ValueError("A player cannot be all-in and folded")

    out = {}
    for p, stack in stacks_bb.items():
        if stack <= 0:
            raise ValueError("Stacks must be positive")
        blind = float(blinds_bb[p])
        if not 0 <= blind <= stack:
            raise ValueError("Invalid blind contribution")
        out[p] = float(stack) if p in all_in else blind
    return out

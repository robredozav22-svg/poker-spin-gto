from __future__ import annotations

from dataclasses import dataclass
from random import Random

from .evaluator import compare_seven
from .hands import HAND_CLASSES, combo_to_class, full_deck
from .regret import RegretNode, expected_value


@dataclass(frozen=True)
class HuMccfrResult:
    btn: dict[str, dict[str, float]]
    bb: dict[str, dict[str, float]]
    stack_bb: float
    iterations: int
    seed: int


def _sample_showdown(rng: Random) -> tuple[tuple[str, str], tuple[str, str], tuple[str, ...], int]:
    cards = rng.sample(full_deck(), 9)
    btn = (cards[0], cards[1])
    bb = (cards[2], cards[3])
    board = tuple(cards[4:9])
    cmp = compare_seven(btn + board, bb + board)
    return btn, bb, board, cmp


def solve_hu_pushfold_chance_sampled(
    stack_bb: float,
    iterations: int = 1_000_000,
    seed: int = 20260906,
) -> HuMccfrResult:
    """Chance-sampled regret solver for HU BTN(SB) jam/fold vs BB call/fold.

    This solver has no external equity table. Every iteration independently
    samples a legal deal and a legal five-card runout, evaluates showdown with
    our own Hold'em evaluator, and updates the sampled private-hand infosets.

    It is deliberately labelled an approximation: the action tree is only
    jam/fold vs call/fold and convergence must be measured empirically.
    """
    if stack_bb <= 1:
        raise ValueError("stack_bb must be > 1")
    if iterations <= 0:
        raise ValueError("iterations must be positive")

    rng = Random(seed)
    btn_nodes = {h: RegretNode(("fold", "jam")) for h in HAND_CLASSES}
    bb_nodes = {h: RegretNode(("fold", "call")) for h in HAND_CLASSES}

    for _ in range(iterations):
        btn_combo, bb_combo, _board, cmp = _sample_showdown(rng)
        btn_hand = combo_to_class(btn_combo)
        bb_hand = combo_to_class(bb_combo)

        btn_node = btn_nodes[btn_hand]
        bb_node = bb_nodes[bb_hand]
        btn_strategy = btn_node.current_strategy()
        bb_strategy = bb_node.current_strategy()

        # Called all-in chip payoff relative to each player's starting stack.
        if cmp > 0:
            btn_showdown = stack_bb
            bb_showdown = -stack_bb
        elif cmp < 0:
            btn_showdown = -stack_bb
            bb_showdown = stack_bb
        else:
            btn_showdown = 0.0
            bb_showdown = 0.0

        # BTN posted 0.5bb. Folding loses 0.5bb. If BB folds to the jam,
        # BTN wins the posted 1bb big blind net of returning own stake: +1bb.
        btn_jam_ev = (1.0 - bb_strategy["call"]) * 1.0 + bb_strategy["call"] * btn_showdown
        btn_values = {"fold": -0.5, "jam": btn_jam_ev}
        btn_ev = expected_value(btn_strategy, btn_values)
        btn_node.add_regrets(btn_values, btn_ev)
        # BTN has no prior own action, so own reach is 1 at the root infoset.
        btn_node.accumulate_strategy()

        # BB's regret update is counterfactually weighted by opponent reach:
        # the BTN jam probability. But average-strategy accumulation is
        # weighted by BB's *own* reach, which is 1 because BB has no earlier
        # action. Using BTN jam probability here would bias the reported
        # average strategy toward periods when BTN happened to jam more often.
        bb_values = {"fold": -1.0, "call": bb_showdown}
        bb_ev = expected_value(bb_strategy, bb_values)
        opponent_reach = btn_strategy["jam"]
        bb_node.add_regrets(bb_values, bb_ev, weight=opponent_reach)
        bb_node.accumulate_strategy()

    return HuMccfrResult(
        btn={h: btn_nodes[h].average_strategy() for h in HAND_CLASSES},
        bb={h: bb_nodes[h].average_strategy() for h in HAND_CLASSES},
        stack_bb=stack_bb,
        iterations=iterations,
        seed=seed,
    )

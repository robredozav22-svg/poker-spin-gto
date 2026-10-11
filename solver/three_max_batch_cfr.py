from __future__ import annotations

from dataclasses import dataclass
from random import Random

from .evaluator import evaluate_seven
from .hands import HAND_CLASSES, combo_to_class, expand_hand_class, full_deck
from .regret import RegretNode, expected_value
from .three_max_mccfr import NODE_ACTIONS, PLAYERS, _terminal_spec, _node_key, public_node_id, terminal_payoff


@dataclass(frozen=True)
class FrozenDeal:
    classes: dict[str, str]
    ranks: dict[str, tuple[int, ...]]


@dataclass(frozen=True)
class ThreeMaxBatchResult:
    stack_bb: float
    sweeps: int
    corpus_seed: int
    samples_per_hand: int
    chance_samples: int
    strategies: dict[str, dict[str, dict[str, float]]]


def build_frozen_corpus(samples_per_hand: int, seed: int) -> dict[tuple[str, str], tuple[FrozenDeal, ...]]:
    """Build a blocker-aware fixed chance corpus for every traverser/hand class.

    The corpus is generated once and reused on every CFR sweep. This removes
    iteration-to-iteration chance noise. A second independently seeded corpus
    must be used for out-of-sample validation; convergence on one frozen corpus
    alone is not certification of the real game.
    """
    if samples_per_hand <= 0:
        raise ValueError("samples_per_hand must be positive")
    rng = Random(seed)
    deck = full_deck()
    out: dict[tuple[str, str], tuple[FrozenDeal, ...]] = {}
    for traverser in PLAYERS:
        for hand in HAND_CLASSES:
            deals: list[FrozenDeal] = []
            combos_for_class = expand_hand_class(hand)
            for _ in range(samples_per_hand):
                hero_combo = rng.choice(combos_for_class)
                used = set(hero_combo)
                remaining = [c for c in deck if c not in used]
                sampled = rng.sample(remaining, 9)
                combos: dict[str, tuple[str, str]] = {traverser: hero_combo}
                others = [p for p in PLAYERS if p != traverser]
                combos[others[0]] = (sampled[0], sampled[1])
                combos[others[1]] = (sampled[2], sampled[3])
                board = tuple(sampled[4:9])
                classes = {p: combo_to_class(combos[p]) for p in PLAYERS}
                ranks = {p: evaluate_seven(combos[p] + board) for p in PLAYERS}
                deals.append(FrozenDeal(classes=classes, ranks=ranks))
            out[(traverser, hand)] = tuple(deals)
    return out


def solve_three_max_pushfold_batch_cfr(
    stack_bb: float,
    sweeps: int = 300,
    corpus_seed: int = 20260916,
    samples_per_hand: int = 8,
    regret_floor: bool = True,
) -> ThreeMaxBatchResult:
    """Deterministic alternating CFR over a fixed blocker-aware chance corpus.

    Key difference from the earlier MCCFR experiments:
    - chance outcomes are frozen across iterations;
    - at opponent nodes *all* legal actions are enumerated and weighted by the
      opponent's current strategy instead of sampling a single action;
    - only the traverser's infosets receive regret updates on that traversal.

    This solves the empirical fixed-corpus game, not exact 3-max Hold'em.
    Independent-corpus validation remains mandatory before any promotion.
    """
    if stack_bb <= 1.0:
        raise ValueError("stack_bb must be > 1")
    if sweeps <= 0:
        raise ValueError("sweeps must be positive")

    corpus = build_frozen_corpus(samples_per_hand=samples_per_hand, seed=corpus_seed)
    nodes: dict[tuple[tuple[str, ...], str, str], RegretNode] = {}
    for history, (actor, actions) in NODE_ACTIONS.items():
        for hand in HAND_CLASSES:
            nodes[_node_key(history, actor, hand)] = RegretNode(actions)

    def traverse(
        history: tuple[str, ...],
        traverser: str,
        classes: dict[str, str],
        ranks: dict[str, tuple[int, ...]],
        own_reach: float,
    ) -> float:
        if _terminal_spec(history, stack_bb) is not None:
            return terminal_payoff(history, stack_bb, ranks)[traverser]

        actor, actions = NODE_ACTIONS[history]
        node = nodes[_node_key(history, actor, classes[actor])]
        strategy = node.current_strategy()

        if actor == traverser:
            action_values: dict[str, float] = {}
            for action in actions:
                action_values[action] = traverse(
                    history + (f"{actor}:{action}",),
                    traverser,
                    classes,
                    ranks,
                    own_reach * strategy[action],
                )
            node_value = expected_value(strategy, action_values)
            node.add_regrets(action_values, node_value)
            if regret_floor:
                for action in node.actions:
                    if node.regret_sum[action] < 0.0:
                        node.regret_sum[action] = 0.0
            node.accumulate_strategy(reach_weight=own_reach)
            return node_value

        # Enumerate all opponent actions. This removes action-sampling variance.
        value = 0.0
        for action in actions:
            value += strategy[action] * traverse(
                history + (f"{actor}:{action}",),
                traverser,
                classes,
                ranks,
                own_reach,
            )
        return value

    for _ in range(sweeps):
        for traverser in PLAYERS:
            for hand in HAND_CLASSES:
                deals = corpus[(traverser, hand)]
                # Each fixed deal is an equally weighted chance sample for this
                # conditioned traverser infoset.
                for deal in deals:
                    traverse((), traverser, deal.classes, deal.ranks, 1.0)

    strategies: dict[str, dict[str, dict[str, float]]] = {}
    for history, (actor, _actions) in NODE_ACTIONS.items():
        strategies[public_node_id(history)] = {
            hand: nodes[_node_key(history, actor, hand)].average_strategy()
            for hand in HAND_CLASSES
        }

    return ThreeMaxBatchResult(
        stack_bb=float(stack_bb),
        sweeps=sweeps,
        corpus_seed=corpus_seed,
        samples_per_hand=samples_per_hand,
        chance_samples=len(PLAYERS) * len(HAND_CLASSES) * samples_per_hand,
        strategies=strategies,
    )

from __future__ import annotations

from dataclasses import dataclass
from random import Random

from .evaluator import evaluate_seven
from .hands import HAND_CLASSES, combo_to_class, expand_hand_class, full_deck
from .regret import RegretNode, expected_value
from .three_max_mccfr import NODE_ACTIONS, PLAYERS, _terminal_spec, _node_key, public_node_id, terminal_payoff


@dataclass(frozen=True)
class ThreeMaxExternalResult:
    stack_bb: float
    sweeps: int
    seed: int
    chance_samples: int
    strategies: dict[str, dict[str, dict[str, float]]]


def _draw_conditioned_on_traverser(
    rng: Random,
    traverser: str,
    traverser_hand: str,
):
    combo = rng.choice(expand_hand_class(traverser_hand))
    used = set(combo)
    remaining = [c for c in full_deck() if c not in used]
    sampled = rng.sample(remaining, 9)

    combos: dict[str, tuple[str, str]] = {traverser: combo}
    others = [p for p in PLAYERS if p != traverser]
    combos[others[0]] = (sampled[0], sampled[1])
    combos[others[1]] = (sampled[2], sampled[3])
    board = tuple(sampled[4:9])
    classes = {p: combo_to_class(combos[p]) for p in PLAYERS}
    ranks = {p: evaluate_seven(combos[p] + board) for p in PLAYERS}
    return classes, ranks


def _sample_action(rng: Random, strategy: dict[str, float]) -> str:
    x = rng.random()
    acc = 0.0
    last = next(iter(strategy))
    for action, p in strategy.items():
        last = action
        acc += p
        if x <= acc:
            return action
    return last


def solve_three_max_pushfold_external_stratified(
    stack_bb: float,
    sweeps: int = 1_000,
    seed: int = 20260912,
) -> ThreeMaxExternalResult:
    """Experimental stratified external-sampling MCCFR for the 3-max push/fold tree.

    A sweep visits every one of the 169 private hand classes once for each
    traversing player. The traverser's concrete combo is sampled uniformly
    inside that class; opponent cards and the board are sampled from the real
    remaining deck. At traverser nodes all actions are explored; at opponent
    nodes one action is sampled from the current strategy.

    This greatly increases per-infoset sample coverage relative to naive
    whole-deal chance sampling. It remains `SOLVER_APPROX`: the public action
    tree is still only shove/fold/call, and multiplayer MCCFR convergence must
    be measured empirically rather than assumed.
    """
    if stack_bb <= 1.0:
        raise ValueError("stack_bb must be > 1")
    if sweeps <= 0:
        raise ValueError("sweeps must be positive")

    nodes: dict[tuple[tuple[str, ...], str, str], RegretNode] = {}
    for history, (actor, actions) in NODE_ACTIONS.items():
        for hand in HAND_CLASSES:
            nodes[_node_key(history, actor, hand)] = RegretNode(actions)

    rng = Random(seed)
    chance_samples = 0

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
                next_history = history + (f"{actor}:{action}",)
                action_values[action] = traverse(
                    next_history,
                    traverser,
                    classes,
                    ranks,
                    own_reach * strategy[action],
                )
            node_value = expected_value(strategy, action_values)
            # External sampling already samples opponents according to their
            # reach probabilities, so no additional opponent-reach multiplier
            # is applied to the sampled regret estimate.
            node.add_regrets(action_values, node_value)
            node.accumulate_strategy(reach_weight=own_reach)
            return node_value

        sampled_action = _sample_action(rng, strategy)
        next_history = history + (f"{actor}:{sampled_action}",)
        return traverse(next_history, traverser, classes, ranks, own_reach)

    for _ in range(sweeps):
        for traverser in PLAYERS:
            for hand in HAND_CLASSES:
                classes, ranks = _draw_conditioned_on_traverser(rng, traverser, hand)
                chance_samples += 1
                traverse((), traverser, classes, ranks, 1.0)

    strategies: dict[str, dict[str, dict[str, float]]] = {}
    for history, (actor, _actions) in NODE_ACTIONS.items():
        strategies[public_node_id(history)] = {
            hand: nodes[_node_key(history, actor, hand)].average_strategy()
            for hand in HAND_CLASSES
        }

    return ThreeMaxExternalResult(
        stack_bb=float(stack_bb),
        sweeps=sweeps,
        seed=seed,
        chance_samples=chance_samples,
        strategies=strategies,
    )

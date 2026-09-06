from __future__ import annotations

from dataclasses import dataclass
from random import Random

from .dcfr import DiscountedRegretNode
from .evaluator import evaluate_seven
from .hands import HAND_CLASSES, combo_to_class, expand_hand_class, full_deck
from .three_max_mccfr import NODE_ACTIONS, PLAYERS, _terminal_spec, _node_key, public_node_id, terminal_payoff


@dataclass(frozen=True)
class ThreeMaxExternalDcfrResult:
    stack_bb: float
    sweeps: int
    seed: int
    chance_samples: int
    strategies: dict[str, dict[str, dict[str, float]]]


def _draw_conditioned(rng: Random, traverser: str, hand: str):
    combo = rng.choice(expand_hand_class(hand))
    used = set(combo)
    rest = [c for c in full_deck() if c not in used]
    sample = rng.sample(rest, 9)
    others = [p for p in PLAYERS if p != traverser]
    combos = {
        traverser: combo,
        others[0]: (sample[0], sample[1]),
        others[1]: (sample[2], sample[3]),
    }
    board = tuple(sample[4:9])
    classes = {p: combo_to_class(combos[p]) for p in PLAYERS}
    ranks = {p: evaluate_seven(combos[p] + board) for p in PLAYERS}
    return classes, ranks


def _sample_action(rng: Random, strategy: dict[str, float]) -> str:
    x = rng.random()
    total = 0.0
    last = next(iter(strategy))
    for action, probability in strategy.items():
        last = action
        total += probability
        if x <= total:
            return action
    return last


def solve_three_max_pushfold_external_dcfr(
    stack_bb: float,
    sweeps: int = 1_000,
    seed: int = 20260914,
    *,
    alpha: float = 1.5,
    beta: float = 0.0,
    gamma: float = 2.0,
    regret_floor: bool = False,
) -> ThreeMaxExternalDcfrResult:
    """Stratified external-sampling DCFR for the restricted 3-max push/fold tree.

    Each sweep conditions on every traverser's 169 hand classes. Opponent
    actions are externally sampled; all traverser actions are enumerated.
    DCFR discounts historical regrets/average mass so noisy early samples have
    less long-run influence. Output remains SOLVER_APPROX until measured stable.
    """
    if stack_bb <= 1.0:
        raise ValueError("stack_bb must be > 1")
    if sweeps <= 0:
        raise ValueError("sweeps must be positive")

    nodes: dict[tuple[tuple[str, ...], str, str], DiscountedRegretNode] = {}
    for history, (_actor, actions) in NODE_ACTIONS.items():
        for hand in HAND_CLASSES:
            nodes[_node_key(history, _actor, hand)] = DiscountedRegretNode(
                actions,
                alpha=alpha,
                beta=beta,
                gamma=gamma,
                regret_floor=regret_floor,
            )

    rng = Random(seed)
    chance_samples = 0

    def traverse(
        history: tuple[str, ...],
        traverser: str,
        classes: dict[str, str],
        ranks: dict[str, tuple[int, ...]],
        own_reach: float,
        iteration: int,
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
                    iteration,
                )
            node_value = sum(strategy[a] * action_values[a] for a in actions)
            node.add_regrets(action_values, node_value, iteration=iteration)
            node.accumulate_strategy(iteration=iteration, reach_weight=own_reach)
            return node_value

        sampled = _sample_action(rng, strategy)
        return traverse(
            history + (f"{actor}:{sampled}",),
            traverser,
            classes,
            ranks,
            own_reach,
            iteration,
        )

    for sweep in range(1, sweeps + 1):
        for traverser in PLAYERS:
            for hand in HAND_CLASSES:
                classes, ranks = _draw_conditioned(rng, traverser, hand)
                chance_samples += 1
                traverse((), traverser, classes, ranks, 1.0, sweep)

    strategies: dict[str, dict[str, dict[str, float]]] = {}
    for history, (actor, _actions) in NODE_ACTIONS.items():
        strategies[public_node_id(history)] = {
            hand: nodes[_node_key(history, actor, hand)].average_strategy()
            for hand in HAND_CLASSES
        }

    return ThreeMaxExternalDcfrResult(
        stack_bb=float(stack_bb),
        sweeps=sweeps,
        seed=seed,
        chance_samples=chance_samples,
        strategies=strategies,
    )

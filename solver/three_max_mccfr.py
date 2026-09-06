from __future__ import annotations

from dataclasses import dataclass
from random import Random
from typing import Callable

from .evaluator import evaluate_seven
from .hands import HAND_CLASSES, combo_to_class, full_deck
from .pots import settle_pots
from .regret import RegretNode, expected_value

PLAYERS = ("BTN", "SB", "BB")

# Public histories and legal action abstractions.
NODE_ACTIONS: dict[tuple[str, ...], tuple[str, tuple[str, ...]]] = {
    (): ("BTN", ("fold", "jam")),
    ("BTN:fold",): ("SB", ("fold", "jam")),
    ("BTN:fold", "SB:jam"): ("BB", ("fold", "call")),
    ("BTN:jam",): ("SB", ("fold", "call")),
    ("BTN:jam", "SB:fold"): ("BB", ("fold", "call")),
    ("BTN:jam", "SB:call"): ("BB", ("fold", "call")),
}


@dataclass(frozen=True)
class ThreeMaxMccfrResult:
    stack_bb: float
    iterations: int
    seed: int
    strategies: dict[str, dict[str, dict[str, float]]]


def public_node_id(history: tuple[str, ...]) -> str:
    if not history:
        return "BTN_ROOT"
    return ">".join(token.replace(":", "_").upper() for token in history)


def _terminal_spec(history: tuple[str, ...], stack_bb: float):
    s = float(stack_bb)
    specs = {
        ("BTN:fold", "SB:fold"): (
            {"BTN": 0.0, "SB": 0.5, "BB": 1.0},
            {"BTN": True, "SB": True, "BB": False},
        ),
        ("BTN:fold", "SB:jam", "BB:fold"): (
            {"BTN": 0.0, "SB": s, "BB": 1.0},
            {"BTN": True, "SB": False, "BB": True},
        ),
        ("BTN:fold", "SB:jam", "BB:call"): (
            {"BTN": 0.0, "SB": s, "BB": s},
            {"BTN": True, "SB": False, "BB": False},
        ),
        ("BTN:jam", "SB:fold", "BB:fold"): (
            {"BTN": s, "SB": 0.5, "BB": 1.0},
            {"BTN": False, "SB": True, "BB": True},
        ),
        ("BTN:jam", "SB:fold", "BB:call"): (
            {"BTN": s, "SB": 0.5, "BB": s},
            {"BTN": False, "SB": True, "BB": False},
        ),
        ("BTN:jam", "SB:call", "BB:fold"): (
            {"BTN": s, "SB": s, "BB": 1.0},
            {"BTN": False, "SB": False, "BB": True},
        ),
        ("BTN:jam", "SB:call", "BB:call"): (
            {"BTN": s, "SB": s, "BB": s},
            {"BTN": False, "SB": False, "BB": False},
        ),
    }
    return specs.get(history)


def terminal_payoff(
    history: tuple[str, ...],
    stack_bb: float,
    ranks: dict[str, tuple[int, ...]],
) -> dict[str, float]:
    spec = _terminal_spec(history, stack_bb)
    if spec is None:
        raise ValueError(f"Not a terminal history: {history}")
    contributions, folded = spec
    live_ranks = {p: ranks[p] for p in PLAYERS if not folded[p]}
    return settle_pots(contributions, folded, live_ranks)


def _sample_deal(rng: Random):
    cards = rng.sample(full_deck(), 11)
    combos = {
        "BTN": (cards[0], cards[1]),
        "SB": (cards[2], cards[3]),
        "BB": (cards[4], cards[5]),
    }
    board = tuple(cards[6:11])
    classes = {p: combo_to_class(combos[p]) for p in PLAYERS}
    ranks = {p: evaluate_seven(combos[p] + board) for p in PLAYERS}
    return classes, ranks


def _node_key(history: tuple[str, ...], actor: str, hand_class: str):
    return history, actor, hand_class


def solve_three_max_pushfold_chance_sampled(
    stack_bb: float,
    iterations: int = 1_000_000,
    seed: int = 20260906,
) -> ThreeMaxMccfrResult:
    """Solve an equal-stack 3-max shove/fold abstraction with chance-sampled CFR.

    Tree:
      BTN: fold / jam
      after BTN fold -> SB: fold / jam -> BB: fold / call
      after BTN jam  -> SB: fold / call -> BB: fold / call

    Every iteration samples all three private hands plus the runout using our
    own deck and evaluator. All action branches for that sampled chance state
    are traversed, including two-way and three-way showdowns.

    IMPORTANT: this is not the full Spin tree. SB limp/raise, BTN minraise and
    postflop continuations are absent, so outputs stay `SOLVER_APPROX`.
    """
    if stack_bb <= 1.0:
        raise ValueError("stack_bb must be > 1")
    if iterations <= 0:
        raise ValueError("iterations must be positive")

    nodes: dict[tuple[tuple[str, ...], str, str], RegretNode] = {}
    for history, (actor, actions) in NODE_ACTIONS.items():
        for hand in HAND_CLASSES:
            nodes[_node_key(history, actor, hand)] = RegretNode(actions)

    rng = Random(seed)

    def traverse(
        history: tuple[str, ...],
        classes: dict[str, str],
        ranks: dict[str, tuple[int, ...]],
        reach: dict[str, float],
    ) -> dict[str, float]:
        terminal = _terminal_spec(history, stack_bb)
        if terminal is not None:
            return terminal_payoff(history, stack_bb, ranks)

        if history not in NODE_ACTIONS:
            raise AssertionError(f"Unknown public history: {history}")
        actor, actions = NODE_ACTIONS[history]
        node = nodes[_node_key(history, actor, classes[actor])]
        strategy = node.current_strategy()

        action_values: dict[str, dict[str, float]] = {}
        for action in actions:
            next_reach = dict(reach)
            next_reach[actor] *= strategy[action]
            next_history = history + (f"{actor}:{action}",)
            action_values[action] = traverse(next_history, classes, ranks, next_reach)

        node_values = {
            p: sum(strategy[a] * action_values[a][p] for a in actions)
            for p in PLAYERS
        }

        # Counterfactual regret is weighted by the reach of all *other*
        # players. Chance reach is uniformly sampled and therefore omitted as
        # the same positive scalar across updates.
        opponent_reach = 1.0
        for p in PLAYERS:
            if p != actor:
                opponent_reach *= reach[p]
        actor_action_values = {a: action_values[a][actor] for a in actions}
        node.add_regrets(actor_action_values, node_values[actor], weight=opponent_reach)

        # Average strategy uses the acting player's own reach probability.
        node.accumulate_strategy(reach_weight=reach[actor])
        return node_values

    for _ in range(iterations):
        classes, ranks = _sample_deal(rng)
        traverse((), classes, ranks, {p: 1.0 for p in PLAYERS})

    strategies: dict[str, dict[str, dict[str, float]]] = {}
    for history, (actor, _actions) in NODE_ACTIONS.items():
        node_id = public_node_id(history)
        strategies[node_id] = {
            hand: nodes[_node_key(history, actor, hand)].average_strategy()
            for hand in HAND_CLASSES
        }

    return ThreeMaxMccfrResult(
        stack_bb=float(stack_bb),
        iterations=iterations,
        seed=seed,
        strategies=strategies,
    )

from __future__ import annotations

from dataclasses import dataclass, field


@dataclass
class RegretNode:
    actions: tuple[str, ...]
    regret_sum: dict[str, float] = field(default_factory=dict)
    strategy_sum: dict[str, float] = field(default_factory=dict)

    def __post_init__(self) -> None:
        if not self.actions:
            raise ValueError("RegretNode needs at least one action")
        for action in self.actions:
            self.regret_sum.setdefault(action, 0.0)
            self.strategy_sum.setdefault(action, 0.0)

    def current_strategy(self) -> dict[str, float]:
        positive = {a: max(0.0, self.regret_sum[a]) for a in self.actions}
        total = sum(positive.values())
        if total <= 0:
            p = 1.0 / len(self.actions)
            return {a: p for a in self.actions}
        return {a: positive[a] / total for a in self.actions}

    def accumulate_strategy(self, reach_weight: float = 1.0) -> dict[str, float]:
        strategy = self.current_strategy()
        for action, probability in strategy.items():
            self.strategy_sum[action] += reach_weight * probability
        return strategy

    def add_regrets(self, action_values: dict[str, float], node_value: float, weight: float = 1.0) -> None:
        missing = set(self.actions) - set(action_values)
        if missing:
            raise ValueError(f"Missing action values: {sorted(missing)}")
        for action in self.actions:
            self.regret_sum[action] += weight * (action_values[action] - node_value)

    def average_strategy(self) -> dict[str, float]:
        total = sum(self.strategy_sum.values())
        if total <= 0:
            return self.current_strategy()
        return {a: self.strategy_sum[a] / total for a in self.actions}


def expected_value(strategy: dict[str, float], action_values: dict[str, float]) -> float:
    if set(strategy) != set(action_values):
        raise ValueError("strategy/action_values keys must match")
    return sum(strategy[a] * action_values[a] for a in strategy)

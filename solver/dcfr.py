from __future__ import annotations

from dataclasses import dataclass, field


@dataclass
class DiscountedRegretNode:
    """Discounted CFR node (research implementation).

    Defaults follow the commonly used DCFR(1.5, 0, 2) schedule:
    positive and negative cumulative regrets are discounted differently,
    while old average-strategy mass is discounted with gamma=2 so later
    iterations receive more weight.

    The caller supplies a monotonically increasing global iteration/sweep
    index. This class is solver plumbing only; using it does not by itself
    certify convergence, especially in a 3-player general-sum game.
    """

    actions: tuple[str, ...]
    alpha: float = 1.5
    beta: float = 0.0
    gamma: float = 2.0
    regret_floor: bool = False
    regret_sum: dict[str, float] = field(default_factory=dict)
    strategy_sum: dict[str, float] = field(default_factory=dict)
    _last_discount_iteration: int = 0
    _last_average_iteration: int = 0

    def __post_init__(self) -> None:
        if not self.actions:
            raise ValueError("DiscountedRegretNode needs actions")
        for action in self.actions:
            self.regret_sum.setdefault(action, 0.0)
            self.strategy_sum.setdefault(action, 0.0)

    def current_strategy(self) -> dict[str, float]:
        positive = {a: max(0.0, self.regret_sum[a]) for a in self.actions}
        total = sum(positive.values())
        if total <= 0.0:
            p = 1.0 / len(self.actions)
            return {a: p for a in self.actions}
        return {a: positive[a] / total for a in self.actions}

    @staticmethod
    def _pow_discount(t: int, exponent: float) -> float:
        x = float(t) ** exponent if exponent != 0.0 else 1.0
        return x / (x + 1.0)

    def _discount_regrets_once(self, iteration: int) -> None:
        if iteration <= self._last_discount_iteration:
            return
        pos_factor = self._pow_discount(iteration, self.alpha)
        neg_factor = self._pow_discount(iteration, self.beta)
        for action in self.actions:
            r = self.regret_sum[action]
            self.regret_sum[action] = r * (pos_factor if r > 0.0 else neg_factor)
        self._last_discount_iteration = iteration

    def add_regrets(
        self,
        action_values: dict[str, float],
        node_value: float,
        *,
        iteration: int,
        weight: float = 1.0,
    ) -> None:
        missing = set(self.actions) - set(action_values)
        if missing:
            raise ValueError(f"Missing action values: {sorted(missing)}")
        if iteration <= 0:
            raise ValueError("iteration must be positive")
        self._discount_regrets_once(iteration)
        for action in self.actions:
            new_value = self.regret_sum[action] + weight * (action_values[action] - node_value)
            if self.regret_floor:
                new_value = max(0.0, new_value)
            self.regret_sum[action] = new_value

    def accumulate_strategy(self, *, iteration: int, reach_weight: float = 1.0) -> dict[str, float]:
        if iteration <= 0:
            raise ValueError("iteration must be positive")
        strategy = self.current_strategy()
        if iteration > self._last_average_iteration:
            factor = (float(iteration) / (float(iteration) + 1.0)) ** self.gamma
            for action in self.actions:
                self.strategy_sum[action] *= factor
            self._last_average_iteration = iteration
        for action, probability in strategy.items():
            self.strategy_sum[action] += reach_weight * probability
        return strategy

    def average_strategy(self) -> dict[str, float]:
        total = sum(self.strategy_sum.values())
        if total <= 0.0:
            return self.current_strategy()
        return {a: self.strategy_sum[a] / total for a in self.actions}

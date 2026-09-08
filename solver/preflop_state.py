from __future__ import annotations

from dataclasses import dataclass, field

PLAYERS = ("BTN", "SB", "BB")


@dataclass
class PreflopState:
    stack_bb: float
    invested: dict[str, float] = field(default_factory=lambda: {"BTN": 0.0, "SB": 0.5, "BB": 1.0})
    folded: set[str] = field(default_factory=set)

    @property
    def pot_bb(self) -> float:
        return sum(self.invested.values())

    def behind(self, player: str) -> float:
        return self.stack_bb - self.invested[player]

    def fold(self, player: str) -> None:
        self.folded.add(player)

    def raise_to(self, player: str, total_bb: float) -> None:
        if total_bb <= self.invested[player]:
            raise ValueError("raise_to must increase invested amount")
        if total_bb > self.stack_bb:
            raise ValueError("raise_to exceeds stack")
        self.invested[player] = float(total_bb)

    def call_to(self, player: str, total_bb: float) -> None:
        if total_bb < self.invested[player]:
            raise ValueError("call target below current investment")
        if total_bb > self.stack_bb:
            raise ValueError("call exceeds stack")
        self.invested[player] = float(total_bb)

    def limp(self, player: str) -> None:
        self.call_to(player, 1.0)

    def check(self, player: str) -> None:
        if player != "BB":
            return
        if self.invested[player] != 1.0:
            raise ValueError("BB check only valid with unchanged blind investment")

    def active_players(self) -> list[str]:
        return [p for p in PLAYERS if p not in self.folded]

    def effective_postflop_stack(self) -> float:
        active = self.active_players()
        if len(active) < 2:
            raise ValueError("postflop requires at least two active players")
        return min(self.behind(p) for p in active)

    def spr(self) -> float:
        pot = self.pot_bb
        if pot <= 0:
            raise ValueError("pot must be positive")
        return self.effective_postflop_stack() / pot


def derive_state(stack_bb: float, history: list[str]) -> PreflopState:
    s = PreflopState(stack_bb=float(stack_bb))
    for token in history:
        parts = token.split("_")
        player = parts[0]
        if player not in PLAYERS:
            raise ValueError(f"unknown player in {token}")
        action = "_".join(parts[1:])
        if action == "FOLD":
            s.fold(player)
        elif action == "LIMP":
            s.limp(player)
        elif action == "CHECK":
            s.check(player)
        elif action.startswith("RAISE_TO_"):
            amount = float(action.removeprefix("RAISE_TO_"))
            s.raise_to(player, amount)
        elif action == "CALL":
            target = max(s.invested.values())
            s.call_to(player, target)
        else:
            raise ValueError(f"unsupported action token: {token}")
    return s

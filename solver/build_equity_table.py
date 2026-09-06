from __future__ import annotations

import argparse
import json
from math import sqrt
from pathlib import Path

from .equity import class_vs_class_monte_carlo
from .hands import HAND_CLASSES


def wilson_half_width_95(p: float, n: int) -> float:
    """Approximate 95% half-width for a Bernoulli proportion.

    Equity includes half-ties, so this is only a conservative diagnostic, not
    a formal confidence interval for the exact equity estimator.
    """
    if n <= 0:
        return 1.0
    return 1.96 * sqrt(max(p * (1.0 - p), 0.0) / n)


def load_or_init(path: Path, samples: int, seed: int) -> dict:
    if path.exists():
        data = json.loads(path.read_text(encoding="utf-8"))
        if data.get("meta", {}).get("samples_per_pair") != samples:
            raise ValueError("Existing table uses a different samples_per_pair")
        if data.get("meta", {}).get("seed") != seed:
            raise ValueError("Existing table uses a different seed")
        return data
    return {
        "meta": {
            "status": "SOLVER_APPROX",
            "engine": "SELF_GENERATED_CLASS_EQUITY_MC_V1",
            "samples_per_pair": samples,
            "seed": seed,
            "symmetry_rule": "eq(a,b)=1-eq(b,a); diagonal=0.5",
            "production_approved": False,
        },
        "equity": {h: {} for h in HAND_CLASSES},
        "diagnostics": {},
    }


def build(path: Path, samples: int, seed: int, start_row: int, end_row: int) -> None:
    data = load_or_init(path, samples, seed)
    table = data["equity"]
    diagnostics = data["diagnostics"]

    end_row = min(end_row, len(HAND_CLASSES))
    for i in range(start_row, end_row):
        hero = HAND_CLASSES[i]
        table.setdefault(hero, {})
        table[hero][hero] = 0.5
        diagnostics[f"{hero}|{hero}"] = {"samples": 0, "half_width_95_approx": 0.0}

        for j in range(i + 1, len(HAND_CLASSES)):
            villain = HAND_CLASSES[j]
            if villain in table[hero]:
                continue
            pair_seed = seed + i * 1000 + j
            result = class_vs_class_monte_carlo(hero, villain, samples=samples, seed=pair_seed)
            eq = result.equity
            table[hero][villain] = eq
            table.setdefault(villain, {})[hero] = 1.0 - eq
            diagnostics[f"{hero}|{villain}"] = {
                "samples": result.total,
                "wins": result.wins,
                "ties": result.ties,
                "losses": result.losses,
                "half_width_95_approx": wilson_half_width_95(eq, result.total),
            }

        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(json.dumps(data, ensure_ascii=False, sort_keys=True), encoding="utf-8")
        print(f"completed row {i}: {hero}")


def main() -> None:
    parser = argparse.ArgumentParser(description="Build our own 169x169 Hold'em class equity table")
    parser.add_argument("--output", default="solver/output/equity-169-mc.json")
    parser.add_argument("--samples", type=int, default=5000)
    parser.add_argument("--seed", type=int, default=20260906)
    parser.add_argument("--start-row", type=int, default=0)
    parser.add_argument("--end-row", type=int, default=len(HAND_CLASSES))
    args = parser.parse_args()

    if args.samples < 100:
        raise SystemExit("Refusing samples < 100: too noisy even for development")
    if not 0 <= args.start_row < len(HAND_CLASSES):
        raise SystemExit("Invalid start row")
    if args.end_row <= args.start_row:
        raise SystemExit("end-row must be greater than start-row")

    build(Path(args.output), args.samples, args.seed, args.start_row, args.end_row)


if __name__ == "__main__":
    main()

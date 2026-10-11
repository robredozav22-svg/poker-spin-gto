import json
import tempfile
import unittest
from pathlib import Path

from solver.postflop_solution_reader import (
    PostflopSolutionFormatError,
    read_solution_meta,
    require_continuation_capable,
)


class PostflopSolutionReaderTests(unittest.TestCase):
    def _write(self, payload):
        f = tempfile.NamedTemporaryFile(mode="w", encoding="utf-8", suffix=".json", delete=False)
        json.dump(payload, f)
        f.close()
        return Path(f.name)

    def test_measured_strategy_without_hand_evs_is_rejected_for_continuation(self):
        p = self._write({
            "config": {"board": "Qs Jh 2h", "effective_stack": 13.0, "starting_pot": 4.5},
            "meta": {
                "engine_version": "0.1.0",
                "iterations": 600,
                "exploitability_pct_of_pot": 0.12,
                "exploitability_chips": 0.0054,
                "payoff_unit": "chips"
            },
            "nodes": [{"actions": [{"kind": "Check"}], "combo_count": 1, "strategy": [1.0]}],
            "root_combos": [[{"index": 0, "cards": "AsAh"}], [{"index": 1, "cards": "KsKh"}]]
        })
        meta = read_solution_meta(p)
        self.assertFalse(meta.has_per_hand_evs)
        with self.assertRaises(PostflopSolutionFormatError):
            require_continuation_capable(meta, max_exploitability_pct=0.5)

    def test_future_hand_ev_extension_can_pass(self):
        p = self._write({
            "config": {"board": "Qs Jh 2h"},
            "meta": {
                "engine_version": "0.2.0",
                "iterations": 800,
                "exploitability_pct_of_pot": 0.10,
                "exploitability_chips": 0.004,
            },
            "nodes": [{
                "actions": [{"kind": "Check"}],
                "combo_count": 1,
                "strategy": [1.0],
                "combo_evs": [0.22]
            }],
            "root_combos": [[{"index": 0, "cards": "AsAh"}], [{"index": 1, "cards": "KsKh"}]]
        })
        meta = read_solution_meta(p)
        self.assertTrue(meta.has_per_hand_evs)
        require_continuation_capable(meta, max_exploitability_pct=0.5)

    def test_exploitability_limit_is_enforced(self):
        p = self._write({
            "config": {"board": "Qs Jh 2h"},
            "meta": {
                "engine_version": "0.2.0",
                "iterations": 20,
                "exploitability_pct_of_pot": 3.0,
                "exploitability_chips": 0.1,
            },
            "nodes": [{
                "actions": [{"kind": "Check"}],
                "combo_count": 1,
                "strategy": [1.0],
                "hand_evs": [0.1]
            }],
            "root_combos": [[{"index": 0, "cards": "AsAh"}], [{"index": 1, "cards": "KsKh"}]]
        })
        meta = read_solution_meta(p)
        with self.assertRaises(PostflopSolutionFormatError):
            require_continuation_capable(meta, max_exploitability_pct=0.5)


if __name__ == "__main__":
    unittest.main()

from __future__ import annotations

import argparse
import json
from pathlib import Path

from .reference_compare import compare_btn_root_to_anchor


def main() -> None:
    parser = argparse.ArgumentParser(description="Compare self-generated BTN root output to a reference anchor")
    parser.add_argument("--solver-json", required=True)
    parser.add_argument("--anchors", default="data/source-verified-anchors.json")
    parser.add_argument("--output", required=True)
    args = parser.parse_args()

    solver_payload = json.loads(Path(args.solver_json).read_text(encoding="utf-8"))
    report = compare_btn_root_to_anchor(solver_payload, args.anchors)
    out = Path(args.output)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(report, ensure_ascii=False, indent=2, sort_keys=True), encoding="utf-8")
    print(json.dumps(report, ensure_ascii=False, indent=2))


if __name__ == "__main__":
    main()

# CI chart audit gate

Workflow: `.github/workflows/chart-audit.yml`

It runs the dependency-free chart validator for chart-data/script changes on pushes to `main` / `chat-aligned-v2` and on pull requests.

The workflow is intentionally read-only (`contents: read`).

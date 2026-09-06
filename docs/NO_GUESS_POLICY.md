# No-guess chart policy

The application may represent a legal poker branch even when exact chart data are missing.

It must never turn that missing data into a guessed GTO action.

Allowed labels:
- VERIFIED_EXACT / A
- SOURCE_CONSTRAINED / B
- MODEL_REFERENCE / M
- EXPLOIT / G
- MISSING_EXACT

The UI must make the distinction visible.

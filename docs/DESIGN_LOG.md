# Design Log

Short, timestamped notes capturing decisions and rationale.

## Format

- YYYY-MM-DD: Decision — Rationale
  - Optional: alternatives considered
  - Optional: links to relevant files/PRs

## Entries

- 2026-01-24: MVP GUI uses a 9x9 grid with visible 3x3 boundaries — simple, clear, and easy to extend.
- 2026-01-24: Keyboard input uses digit keys + arrows + delete/backspace — minimal UX with low implementation cost.
- 2026-01-24: Prioritize visibility highlights (row/column/box + same digit) and track digit tally as a backlog item — focus on immediate playability pain points.
- 2026-01-24: Highlight spec — row/column/box use a neutral tint (not necessarily warm); same digit uses a distinct cool tint; cool wins on overlap; apply same-digit highlight only when the selected cell contains a digit, tuned for dark theme readability.

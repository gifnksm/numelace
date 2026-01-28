# Design Log

Short, timestamped notes capturing decisions and rationale.

## Format

- YYYY-MM-DD: Decision — Rationale
  - Note: confirm the current date before adding an entry (use the system datetime tool)
  - Note: append new entries to the end (chronological order)
  - Optional: alternatives considered
  - Optional: links to relevant files/PRs

## Entries

- 2026-01-24: MVP GUI uses a 9x9 grid with visible 3x3 boundaries — simple, clear, and easy to extend.
- 2026-01-24: Keyboard input uses digit keys + arrows + delete/backspace — minimal UX with low implementation cost.
- 2026-01-24: Prioritize visibility highlights (row/column/box + same digit) and track digit tally as a backlog item — focus on immediate playability pain points.
- 2026-01-24: Highlight spec — row/column/box use a neutral tint (not necessarily warm); same digit uses a distinct cool tint; cool wins on overlap; apply same-digit highlight only when the selected cell contains a digit, tuned for dark theme readability.
- 2026-01-24: Number pad UI — 2x5 layout; digits centered; per-digit filled count in the top-right; counts show filled totals; Del clears selected cell — improves mouse-only input while surfacing progress.
- 2026-01-25: UI uses ViewModels and `Action` returns for input/interaction — keeps rendering decoupled from game logic and centralizes action application.
- 2026-01-25: Game exposes can_* helpers for action gating — reduces UI-side rule checks.
- 2026-01-25: Always show the New Game confirmation dialog, even when the puzzle is solved — consistent UX and prevents accidental reset.
- 2026-01-26: Persist app state via eframe storage with RON serialization and DTO/try-from conversions — safe restoration with failure fallback to defaults and periodic + action-triggered saves.
- 2026-01-26: Candidate notes are exclusive with filled digits; memo input is toggle-based; normal input clears memos; memo input on filled cells is ignored; input mode toggles with S and command provides temporary swap with ^ indicator — keeps UX consistent and clear.
- 2026-01-27: UI uses per-cell `content` and `visual state` (selection/house/same-digit/conflict) derived in app; rule-based conflict checks live in `numelace-game`; UI terminology sticks to `house` for consistency — keeps rule logic centralized while keeping UI state explicit and aligned with existing terms.
- 2026-01-27: Use container-level `#[serde(default)]` for DTOs with sensible defaults (and map `Default` from state defaults) so missing fields preserve non-false defaults — keeps deserialization backward compatible.
- 2026-01-27: Skip extra commit confirmation when the user explicitly asks to commit — reduces redundant prompts while keeping confirmation for other cases.
- 2026-01-27: Strict rule checks still allow clearing existing digits/notes — preserves safe undo of inputs while preventing new conflicts.
- 2026-01-27: Strict-conflicting inputs are rejected but shown as ghost UI state — surfaces rule violations without mutating game state.
- 2026-01-28: App logic refactor splits Action handling, view model building, and action request queuing — improves responsibility separation and testability.
- 2026-01-28: Undo/redo uses snapshot history with selection-aware restore and a top toolbar entry point — keeps undoable state consistent and exposes mouse-friendly controls without overloading the keypad area.
- 2026-01-28: Assist auto-removes row/col/box notes on fill (including replacements), defaults enabled, and does nothing on rejected inputs — keeps assist behavior clear and limited to fill actions.
- 2026-01-28: Replace digit input parameters with a `InputDigitOptions` struct (builder-style setters, defaults) — keeps API extensible without piling on flags.
- 2026-01-28: Centralize CellState transitions (fill, note toggle, clear) into CellState methods — keeps state invariants consistent across game logic.

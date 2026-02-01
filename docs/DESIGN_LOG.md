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
- 2026-01-29: Digit input is non-toggle (same digit is no-op); notes remain toggle-based — aligns with typical Sudoku UX and keeps clear-cell as the explicit removal action.
- 2026-01-29: Clarify `Game` vs `CellState` responsibilities (cell-local capability checks in `CellState`, board-level rules and peer effects orchestrated by `Game`) and split `numelace-game` into `game`, `cell_state`, `input`, and `error` modules — keeps local invariants centralized and reduces drift.
- 2026-01-29: Represent input outcomes as `Result<InputOperation, InputBlockReason>` for capability checks and operations — makes no-op/set/remove outcomes explicit while keeping board-level rules in `Game`.
- 2026-01-29: Keypad action indicators show note add/remove via compact icons and the notes toggle uses a dedicated pencil button — improves clarity while reducing visual noise.
- 2026-01-30: README is user-focused while CONTRIBUTING covers developer workflows — keeps entry points clear for each audience.
- 2026-01-30: Provide a local CI script mirroring GitHub Actions checks — makes it easy to run the same checks before pushing.
- 2026-01-30: Use a front-and-center Settings modal (on-demand) and move theme switching to the toolbar — simplifies access while keeping the grid area clean.
- 2026-01-31: Auto-generate `icon.rs` from emoji-icon-font CSS via a script — keeps icon ordering aligned with upstream and reduces manual updates.
- 2026-01-31: Add reset-puzzle action with confirmation, toolbar entry, and shortcut — distinguishes input reset from New Game and reduces accidental loss.
- 2026-01-31: Quantize UI cell_size to 1/100 steps before GUI rounding — prevents cumulative layout drift from rounding.
- 2026-02-01: Notes auto-fill uses keypad (selected cell) + toolbar (all cells), `a`/`A` shortcuts, replaces notes with peer-exclusion candidates computed in `numelace-game`, and defaults auto-fill on new game/reset to ON — keeps UX discoverable while centralizing rule-driven note generation.
- 2026-02-01: Grid rendering now uses a dedicated grid palette/theme structure (light/dark), initially mirroring existing visuals — enables future hint colors and clearer semantic color tuning.

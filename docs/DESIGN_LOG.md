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
- 2026-02-02: Hint steps are derived by per-technique `find_step` returning `TechniqueStep` with condition/application data, while game applies changes — keeps hint derivation non-destructive and consistent with technique logic.
- 2026-02-04: Game persists puzzle solutions for hint placement verification and applies hint candidate eliminations only to existing notes — enables deterministic hint validation while keeping memo updates explicit.
- 2026-02-04: Hint placement validation is performed at the step level by checking `TechniqueStep` placements against the stored solution — keeps validation aligned with hint application and avoids pushing step decomposition logic into the app layer.
- 2026-02-05: Use “inconsistent” for immediate A/B issues and “no solution” for solvability outcomes — separates current rule violations from unsolvable states.
- 2026-02-06: Offload new game generation via a worker-backed async pipeline with a lightweight DTO and UI spinner — keeps egui responsive and supports WASM hosting constraints.
- 2026-02-06: Add a lightweight flow executor to orchestrate UI async flows (e.g., new game confirmation + background work) — keeps control flow linear and avoids scattered state transitions.
- 2026-02-06: Make async work awaitable via flow futures with a spinner wrapper — enables generic flow orchestration while reusing the async_work pipeline.
- 2026-02-06: Route solvability result dialogs through flow awaitables — keeps modal handling consistent with flow-driven async UI.
- 2026-02-06: Track in-flight work via `WorkKind` instead of per-task flags — keeps async work extensible without accumulating booleans.
- 2026-02-07: Route modal flows through per-request oneshot responders — makes modal awaits local and supports concurrent flows.
- 2026-02-07: Route work requests through per-request responders — keeps background work awaits local and ready for concurrency.
- 2026-02-07: Allow multiple in-flight work entries — unblocks concurrent work execution.
- 2026-02-07: Make modal requests carry typed responders — keeps modal responses type-safe without a shared enum.
- 2026-02-07: Simplify async work by removing WorkFlow and routing work futures directly through flow — reduces duplicate polling and keeps work responses handled via actions.
- 2026-02-07: Track app state dirtiness via mutable access and consume work responses in the flow executor — keeps persistence tied to real state mutations while avoiding extra action plumbing.
- 2026-02-07: Rename `flow` to `flow_executor`, `async_work` to `worker`, and move UI flows into `action_handler/flows` — clarifies responsibilities while keeping the executor generic.
- 2026-02-07: Move worker tasks under `worker/tasks` and split `worker/api` out of the worker module — clarifies worker responsibilities and API surface.
- 2026-02-07: Drive spinner UI via `StartSpinner`/`StopSpinner` actions with per-flow IDs and UI-managed active list (oldest active shown) — keeps executor focused on flow orchestration and UI reactive to flow activity.
- 2026-02-07: Persist undo/redo history using compact snapshots (filled digits + notes + selection) and reconstruct game state from problem/solution on restore — reduces memory/serialized size while keeping undo/redo reliable across sessions.
- 2026-02-07: Move undo/redo history ownership into `AppState` (with UI delegation) — simplifies persistence and keeps history tied to core game state.
- 2026-02-07: Show a solvability undo notice dialog with the number of steps undone — clarifies rollback impact for the user.
- 2026-02-07: Consolidate modal dialogs into confirm/alert kinds while keeping typed responses — reduces modal variants without sacrificing type safety.
- 2026-02-07: Restructure flow code into an independent `flow` module with executor, helpers, and tasks — makes flow independence explicit and mirrors worker organization.
- 2026-02-09: Hint flow prompts to rebuild notes when a hint exists only without notes and clears hint state afterward — keeps hint UX aligned with solvability checks and avoids misleading memo-based hints.
- 2026-02-09: Hint stage 1 uses corner marks for highlighting — avoids background conflicts with same-digit highlights and keeps selection visibility.
- 2026-02-09: Hint stage 2 highlights condition digits with pill backgrounds — keeps digit-level emphasis distinct from stage 1 region marking.
- 2026-02-10: Hint stage 3 uses preview/apply sub-steps with mandatory preview visuals (hint-colored outline + digits/notes) and distinct status-line messaging — makes confirmation explicit before applying progress.
- 2026-02-10: Hint state is defined as user-visible presentation, so post-apply feedback may remain briefly while board highlights clear — aligns UX feedback with stage transitions.

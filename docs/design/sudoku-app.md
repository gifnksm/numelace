# sudoku-app Design Draft (Desktop MVP)

## Goals

Provide a minimal but functional desktop GUI for Sudoku that:

- Displays a 9x9 board.
- Allows cell selection and digit input via keyboard.
- Generates a new puzzle on demand.
- Detects and indicates completion using `sudoku-game::Game::is_solved()`.
- Establishes a clear path for future features (number pad, candidate marks, undo/redo, hints).

This design focuses on desktop only. Web/WASM support is deferred.

## Non-Goals (for MVP)

- Candidate marks.
- Undo/redo.
- Hints or mistake detection.
- Save/load or persistence.
- Difficulty selection.

## Crate Dependencies

`crates/sudoku-app` will depend on:

- `sudoku-game` (game state)
- `sudoku-generator` + `sudoku-solver` (puzzle generation)
- `sudoku-core` (shared types like `Digit`, `Position`)
- `eframe` / `egui` (UI)

Update `crates/sudoku-app/Cargo.toml` accordingly.

## UI Layout

Single window layout with a central board:

```text
+------------------------------------------------+
| Sudoku                                         |
|                                                |
|  [9x9 board grid]                               |
|                                                |
|  Status: In progress / Solved                  |
|  [New Game]                                    |
+------------------------------------------------+
```

### Board Rendering

- Render a 9x9 grid using egui (e.g., `Grid`, `Table`, or custom painting).
- Distinguish cell borders to show 3x3 subgrids (thicker lines).
- Each cell shows:
  - Given digit: bold and darker.
  - Player digit: normal weight.
  - Empty: blank.

### Selection

- One active cell at a time.
- Selected cell is highlighted (background color).
- Clicking a cell selects it (if allowed by UI; no restriction on given cells for selection).

### Keyboard Input

- Digit keys `1`–`9`:
  - If the selected cell is not a given cell:
    - `Game::set_digit` with corresponding `Digit`.
  - Given cell: ignore input (optional: show brief status message).
- `Backspace` or `Delete`:
  - `Game::clear_cell` if not a given cell.
- Arrow keys:
  - Move selection in the grid (clamp to bounds).
- `N` or `Ctrl+N`:
  - Start a new game (optional shortcut).
- `Esc`:
  - Clear selection (optional).

### Status Display

- Text label below board:
  - "In progress" while not solved.
  - "Solved!" when `game.is_solved()` is true.

## App State Design

Introduce an app struct in `sudoku-app`:

```rust
struct SudokuApp {
    game: Game,
    selected: Option<Position>,
    status: String, // optional for errors / info
}
```

### Initialization

- Create `TechniqueSolver::with_all_techniques()`.
- Create `PuzzleGenerator::new(&solver)`.
- Generate puzzle.
- Create `Game::new(puzzle)`.

### State Updates

- On New Game:
  - Generate a new puzzle and replace `game`.
  - Clear `selected`.
  - Update `status`.

## Rendering Strategy (egui)

- Use `CentralPanel` to render UI.
- Use a fixed-size grid for board cells (e.g., `egui::Grid` or custom layout).
- Compute `Position` from row/col.
- For each cell:
  - Render a clickable button or label (prefer a button-like widget for selection).
  - Apply background color if selected.
  - Apply different text styles for given vs filled.

## Error Handling

- `Game::set_digit` and `Game::clear_cell` can return `GameError::CannotModifyGivenCell`.
- For MVP:
  - Ignore the error silently.
  - Optionally set a short `status` message.

## Extensibility Notes

This MVP should not block future features:

- **Number Pad UI**: Add a row of digit buttons below the board that trigger the same input logic.
- **Candidate Marks**: Add a cell overlay mode for small digits.
- **Undo/Redo**: Maintain a history stack in app state.
- **Hints/Mistakes**: Use `sudoku-solver` to suggest or validate.
- **WASM**: separate entrypoint or feature flags later.

## Testing

- UI is typically not unit-tested, but:
  - Keep `SudokuApp` logic small and pure where possible.
  - Extract input handling into methods that can be tested separately.
- Existing `sudoku-game` tests are sufficient for logic correctness.

## Decisions

- Difficulty selection: not included in MVP.
- Window size: set an initial size, allow resizing, and enforce a minimum size (preferred balance of UX and layout stability).
- Completion display: text-only status (no modal).

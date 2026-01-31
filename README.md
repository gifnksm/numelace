# Numelace

Numelace is a number-place (Sudoku) puzzle application written in Rust, with desktop and Web/WASM support.

## Project Goals

- **Automatic Puzzle Generation**: Generate Sudoku puzzles with configurable difficulty levels
- **Multiple Solving Strategies**: Implement both algorithmic (backtracking) and human-like solving techniques
- **Cross-Platform**: Desktop and Web/WASM GUIs using egui/eframe

## Current Status

Planned features are tracked in docs/BACKLOG.md.

- Gameplay:
  - ✅ 9x9 board with clear 3x3 boundaries
  - ✅ Core rules: given vs filled cells and solved-state validation
  - ✅ Candidate notes (player notes)
  - ✅ Undo/redo
- Puzzle & solver:
  - ✅ Puzzle generation with unique solutions, reproducible seeds, and technique-solver solvability (human-style techniques)
  - ✅ Solver with basic techniques (Naked/Hidden Single) plus backtracking
- Optional assist features:
  - ✅ Highlight toggles (same digit, house, conflicts)
  - ✅ Mistake highlighting (row/col/box conflicts)
  - ✅ Rule-violation blocking with ghost preview (shows attempted inputs without applying them)
  - ✅ Blocked-candidate indicator on keypad buttons (toggleable)
  - ✅ Auto-remove row/col/box notes on fill
- Application features:
  - ✅ Desktop GUI (keypad with digit counts, theme toggle, new game confirmation)
  - ✅ Assist toggles UI (optional features on/off)
  - ✅ Auto-save and resume (board state + settings)
  - ✅ Web/WASM support

## Web Demo

- Web demo: <https://gifnksm.github.io/numelace>

## Run on Desktop

```bash
cargo run --release
```

## Controls

### Input modes

- **Keyboard-only**: all inputs via keys.
- **Mouse-only**: use the on-screen keypad and toolbar buttons.
- **Mixed**: combine keyboard and mouse.

### Keyboard

- **Movement & selection**
  - <kbd>↑</kbd>/<kbd>↓</kbd>/<kbd>←</kbd>/<kbd>→</kbd>: move the selected cell.
  - <kbd>Esc</kbd>: clear selection.
- **Digit entry & notes**
  - <kbd>1</kbd>–<kbd>9</kbd>: enter a digit for the selected cell.
  - <kbd>S</kbd>: toggle between Fill and Notes modes.
  - <kbd>Ctrl</kbd>/<kbd>Cmd</kbd> (hold): temporarily swap Fill/Notes while held.
- **Clearing**
  - <kbd>Delete</kbd>/<kbd>Backspace</kbd>: clear the selected cell (digit or notes).
- **History & game actions**
  - <kbd>Ctrl</kbd>/<kbd>Cmd</kbd>+<kbd>Z</kbd>: undo.
  - <kbd>Ctrl</kbd>/<kbd>Cmd</kbd>+<kbd>Y</kbd>: redo.
  - <kbd>Ctrl</kbd>/<kbd>Cmd</kbd>+<kbd>N</kbd>: new game (confirmation shown).
  - <kbd>Ctrl</kbd>/<kbd>Cmd</kbd>+<kbd>Shift</kbd>+<kbd>Backspace</kbd>: reset puzzle (confirmation shown).
  - <kbd>Ctrl</kbd>/<kbd>Cmd</kbd>+<kbd>,</kbd>: open settings.

### Mouse

- **Keypad**
  - <kbd>1</kbd>–<kbd>9</kbd> + <kbd>Delete</kbd>: entry and clearing.
  - Notes toggle button (pencil): switch input mode.
  - <kbd>Ctrl</kbd>/<kbd>Cmd</kbd> (hold): temporarily swap Fill/Notes for keypad digit buttons.
  - Per-digit counts: show how many of each digit are already placed.
  - Notes mode indicators: digit buttons show note add/remove actions.
- **Toolbar**
  - Undo, Redo, New Game, Reset Puzzle, Settings buttons.
  - Theme switch (Light/Dark/System).

## Project Structure

```text
crates/  # workspace crates
docs/    # project documentation
```

## Documentation

- [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) - architecture and design decisions
- [docs/WORKFLOW.md](docs/WORKFLOW.md) - development workflow
- [docs/BACKLOG.md](docs/BACKLOG.md) - ideas and future work
- [docs/DESIGN_LOG.md](docs/DESIGN_LOG.md) - decision history
- [docs/TESTING.md](docs/TESTING.md) - testing guidelines
- [docs/DOCUMENTATION_GUIDE.md](docs/DOCUMENTATION_GUIDE.md) - documentation conventions

For development workflows and contribution guidelines, see [CONTRIBUTING.md](CONTRIBUTING.md).

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

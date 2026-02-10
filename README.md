# Numelace

Numelace is a number-place (Sudoku) puzzle application written in Rust, with desktop and Web/WASM support.

## Project Goals

- **Automatic Puzzle Generation**: Generate Sudoku puzzles with configurable difficulty levels
- **Multiple Solving Strategies**: Implement both algorithmic (backtracking) and human-like solving techniques
- **Cross-Platform**: Desktop and Web/WASM GUIs using egui/eframe

## Try the Web Demo

<https://gifnksm.github.io/numelace/>

The Web Demo is PWA-ready. You can play on your phone and add it to your home screen.

## Run on Desktop (Local)

```bash
cargo run --release
```

## Current Status

Planned features are tracked in docs/BACKLOG.md.

- Core UX: notes, undo/redo, highlight toggles, rule-violation preview
- Platforms: Desktop + Web/WASM
- Persistence: auto-save and resume
- UI: on-screen keypad, theme switch, settings modal
- Tools: solvability check

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
  - <kbd>A</kbd>: auto-fill notes for the selected cell.
  - <kbd>Shift</kbd>+<kbd>A</kbd>: auto-fill notes for all cells.
- **Clearing**
  - <kbd>Delete</kbd>/<kbd>Backspace</kbd>: clear the selected cell (digit or notes).
- **History & game actions**
  - <kbd>Ctrl</kbd>/<kbd>Cmd</kbd>+<kbd>Z</kbd>: undo.
  - <kbd>Ctrl</kbd>/<kbd>Cmd</kbd>+<kbd>Y</kbd>: redo.
  - <kbd>Ctrl</kbd>/<kbd>Cmd</kbd>+<kbd>N</kbd>: new game (confirmation shown).
  - <kbd>Ctrl</kbd>/<kbd>Cmd</kbd>+<kbd>Shift</kbd>+<kbd>Backspace</kbd>: reset inputs (confirmation shown).
  - <kbd>Ctrl</kbd>/<kbd>Cmd</kbd>+<kbd>,</kbd>: open settings.
  - <kbd>Ctrl</kbd>/<kbd>Cmd</kbd>+<kbd>K</kbd>: check solvability.
  - <kbd>Ctrl</kbd>/<kbd>Cmd</kbd>+<kbd>J</kbd>: hint (press repeatedly to reveal details and apply the next step).

### Mouse

- **Keypad**
  - <kbd>1</kbd>–<kbd>9</kbd> + <kbd>Delete</kbd>: entry and clearing.
  - Notes toggle button (pencil): switch input mode.
  - Auto-fill notes button (selected cell).
  - <kbd>Ctrl</kbd>/<kbd>Cmd</kbd> (hold): temporarily swap Fill/Notes for keypad digit buttons.
  - Per-digit counts: show how many of each digit are already placed.
  - Notes mode indicators: digit buttons show note add/remove actions.
- **Toolbar**
  - Undo, Redo, Auto-fill notes (all cells), Check Solvability, Hint, New Game, Reset Inputs, Settings buttons.
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

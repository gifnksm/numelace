# Numelace

Numelace is a number-place (Sudoku) puzzle application written in Rust, supporting both desktop and web platforms.

## Project Goals

- **Automatic Puzzle Generation**: Generate Sudoku puzzles with configurable difficulty levels
- **Multiple Solving Strategies**: Implement both algorithmic (backtracking) and human-like solving techniques
- **Cross-Platform**: Desktop GUI and Web/WASM support using egui/eframe

## Current Status

- ✅ **numelace-core**: Core data structures **implemented**
  - Type-safe grid containers and indexing (CandidateGrid, DigitGrid)
  - Basic types (Digit, Position) with semantic indexing

- ⚙️ **numelace-solver**: Solver framework **implemented** (techniques: minimal)
  - Technique-based solver and backtracking solver
  - Current: basic techniques (Naked/Hidden Single)
  - TODO: Naked/Hidden Pairs, Pointing Pairs, Box/Line Reduction, X-Wing, etc.

- ✅ **numelace-generator**: Puzzle generation **implemented**
  - Removal method with unique solution guarantee
  - Reproducible generation via seeds

- ⚙️ **numelace-game**: Game logic **minimally implemented**
  - Game session management with basic operations
  - TODO: candidate marks, undo/redo, hints, save/load

- ⚙️ **numelace-app**: GUI **minimally implemented**
  - 9x9 board rendering with 3x3 boundaries
  - Mouse selection and keyboard input
  - New game and solved status display

## Project Structure

```text
crates/
├── numelace-core/       # Core data structures (CandidateGrid, DigitGrid, Digit, Position)
├── numelace-solver/     # Solving algorithms (technique-based + backtracking)
├── numelace-generator/  # Puzzle generation
├── numelace-game/       # Game logic
└── numelace-app/        # GUI application (desktop)
```

See [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) for architecture and implementation plans, [docs/TESTING.md](docs/TESTING.md) for testing guidelines, and [docs/TODO.md](docs/TODO.md) for current tasks.

## Build and Run

```bash
# Build all crates
cargo build

# Build with optimizations
cargo build --release

# Run tests
cargo test
```

For development commands (clippy, benchmarks, documentation generation, etc.), see [CONTRIBUTING.md](CONTRIBUTING.md).

```bash
# Desktop application
cargo run --release
```

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

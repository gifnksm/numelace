# Sudoku

A Sudoku application written in Rust, supporting both desktop and web platforms.

## Project Goals

- **Automatic Puzzle Generation**: Generate Sudoku puzzles with configurable difficulty levels
- **Multiple Solving Strategies**: Implement both algorithmic (backtracking) and human-like solving techniques
- **Cross-Platform**: Desktop GUI and Web/WASM support using egui/eframe

## Current Status

- ✅ **sudoku-core**: Core data structures **completed**
  - `Digit`: Type-safe representation of numbers 1-9
  - `Position`: Board coordinates with box calculation utilities
  - `CandidateGrid`: Candidate tracking grid for solving algorithms (digit-centric)
  - `DigitGrid`: Simple cell-centric grid with string parsing/formatting
  - Generic containers (`BitSet9`, `BitSet81`, `Array9`, `Array81`)
  - Type-safe indexing with semantic index types
- 📋 **Next**: Solver algorithms, puzzle generation, GUI

## Project Structure

```text
crates/
├── sudoku-core/       # Core data structures (CandidateGrid, DigitGrid, Digit, Position)
├── sudoku-solver/     # Solving algorithms (planned)
├── sudoku-generator/  # Puzzle generation (planned)
├── sudoku-game/       # Game logic (planned)
└── sudoku-app/        # GUI application (planned)
```

See [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) for detailed architecture documentation.

## Build and Test

```bash
# Build all crates
cargo build

# Run tests
cargo test

# Run clippy
cargo clippy --all-targets

# Generate documentation
cargo doc --open
```

## Run

```bash
# Desktop application (not yet implemented)
cargo run -p sudoku-app
```

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

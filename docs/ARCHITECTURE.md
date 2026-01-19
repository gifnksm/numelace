# Sudoku Application Architecture

## Overview

This document describes the architecture of the Sudoku application, including the crate structure, responsibilities, and dependencies.

## Project Goals

- **Problem Generation**: Automatically generate Sudoku puzzles with configurable difficulty levels
- **Multiple Solving Strategies**: Implement both algorithmic (backtracking) and human-like solving techniques
- **Multi-Platform Support**: Desktop GUI (egui/eframe) and Web (WASM)
- **Interactive Features**: Hints, mistake detection, undo/redo functionality

## Crate Structure

```text
sudoku/
├── crates/
│   ├── sudoku-core/          # Core data structures and types
│   ├── sudoku-solver/        # Solving algorithms (planned)
│   ├── sudoku-generator/     # Puzzle generation (planned)
│   ├── sudoku-game/          # Game logic and state management (planned)
│   └── sudoku-app/           # GUI application (desktop + web)
└── docs/
    └── ARCHITECTURE.md       # This file
```

## Crate Descriptions

### sudoku-core

**Status**: In Development

**Purpose**: Provides fundamental data structures and types for representing Sudoku puzzles.

**Key Components**:

- `NumberSet`: Efficient bitset representation of numbers 1-9 using a 16-bit integer
- `Position`: Row and column coordinates (planned)
- `Cell`: Individual cell with candidate values (planned)
- `Grid`: Complete puzzle state using BitBoard implementation with 128-bit integers (planned)
- Basic rule validation (duplicate checking, etc.) (planned)

**Dependencies**: None

**Design Decisions**:

- BitBoard implementation chosen for performance and memory efficiency
- `NumberSet` uses only bits 0-8 of a u16 to represent numbers 1-9
- All operations are designed to be copy-friendly and cache-efficient

---

### sudoku-solver

**Status**: Planned

**Purpose**: Implements various solving algorithms for Sudoku puzzles.

**Key Components**:

```text
sudoku-solver/
├── backtrack.rs          # Backtracking algorithm (for generation and validation)
├── techniques/
│   ├── mod.rs
│   ├── basic.rs         # Naked Single, Hidden Single
│   ├── intermediate.rs  # Locked Candidates, Naked/Hidden Pairs/Triples
│   └── advanced.rs      # X-Wing, Swordfish, XY-Wing, etc.
└── solver.rs            # Common Solver trait
```

**Solving Strategies**:

1. **Backtracking Solver**:
   - Used for puzzle generation
   - Validates puzzle uniqueness
   - Exhaustive solution finding

2. **Human-like Techniques** (by difficulty):
   - **Basic**: Naked Single, Hidden Single
   - **Intermediate**: Locked Candidates, Naked/Hidden Pairs, Pointing Pairs
   - **Advanced**: X-Wing, Swordfish, XY-Wing, XYZ-Wing
   - **Expert**: Coloring, Chains, Forcing Chains

**Dependencies**: `sudoku-core`

**Design Decisions**:

- Each technique is a separate module for clarity and testability
- Common `Solver` trait allows pluggable solving strategies
- Techniques return applied moves for hint generation and difficulty assessment

---

### sudoku-generator

**Status**: Planned

**Purpose**: Generates valid Sudoku puzzles with specified difficulty levels.

**Key Components**:

- Puzzle generation algorithm using backtracking solver
- Difficulty evaluation based on required solving techniques
- Solution uniqueness verification

**Generation Algorithm**:

1. Generate a complete valid grid using backtracking
2. Remove numbers while maintaining unique solution
3. Evaluate difficulty by attempting to solve with human techniques
4. Adjust number of clues based on target difficulty

**Dependencies**: `sudoku-core`, `sudoku-solver`

---

### sudoku-game

**Status**: Planned

**Purpose**: Manages game state, user interactions, and game logic.

**Key Components**:

- Game state management (current puzzle, solution, progress)
- Undo/Redo stack
- Hint system (suggests next move using solver techniques)
- Mistake detection and validation
- Save/load puzzle state
- Timer and statistics

**Dependencies**: `sudoku-core`, `sudoku-solver`, `sudoku-generator`

---

### sudoku-app

**Status**: In Development

**Purpose**: GUI application for both desktop and web platforms using egui/eframe.

**Key Components**:

- Puzzle board rendering
- Cell selection and input handling
- Menu system (New Game, Difficulty, Settings)
- Visual feedback (highlights, error indicators)
- UI state management
- Application entry point (`main.rs` for desktop)
- WASM support for web deployment
- Configuration management

**Platform Support**:

- **Desktop**: Native application via `cargo run`
- **Web**: WASM compilation via `trunk build` or `wasm-pack`
- eframe provides unified API for both platforms

**Dependencies**: `sudoku-game`, `eframe`

**Design Decisions**:

- Single crate for both desktop and web to avoid premature abstraction (YAGNI principle)
- eframe handles platform differences internally
- Platform-specific code uses conditional compilation when needed
- If significant divergence occurs, can be split later

---

## Dependency Graph

```text
sudoku-core
    ↓
sudoku-solver
    ↓
sudoku-generator
    ↓
sudoku-game
    ↓
sudoku-app (desktop + web)
```

**Principles**:

- Dependencies flow in one direction (no circular dependencies)
- Lower-level crates have no knowledge of higher-level crates
- Core data structures are independent and reusable
- UI implementations depend on game logic, not vice versa

---

## Key Design Decisions

### BitBoard Implementation

**Decision**: Use 128-bit integers for grid representation.

**Rationale**:

- Each cell needs 9 bits (one per candidate number)
- 81 cells × 9 bits = 729 bits minimum
- 128-bit integers provide efficient bitwise operations
- Allows for fast constraint propagation and validation

**Trade-offs**:

- More complex implementation
- Higher memory usage per grid
- Significant performance benefits for solving algorithms

### Separation of Solver Techniques

**Decision**: Each solving technique is implemented as a separate module.

**Rationale**:

- Easy to add new techniques without modifying existing code
- Clear testing boundaries
- Difficulty evaluation based on technique complexity
- Hint system can explain which technique to use

### Solver-Based Generation

**Decision**: Use solving techniques to evaluate puzzle difficulty during generation.

**Rationale**:

- Ensures puzzles are solvable with human techniques
- Difficulty rating matches player experience
- Can target specific technique practice
- Validates that puzzles don't require guessing

---

## Testing Strategy

### Unit Tests

- Each crate has comprehensive unit tests
- Property-based testing for core data structures (proptest)
- Edge case coverage for all public APIs

### Integration Tests

- Solver correctness on known puzzles
- Generator produces valid, unique-solution puzzles
- Game state transitions maintain invariants

### Performance Tests

- Solver performance benchmarks
- Generation speed targets
- Memory usage profiling

---

## References

- [Rust Book](https://doc.rust-lang.org/book/)
- [egui Documentation](https://docs.rs/egui/)
- [Sudoku Solving Techniques](http://www.sudokuwiki.org/sudoku.htm)
- [BitBoard Techniques](https://www.chessprogramming.org/Bitboards)

---

**Last Updated**: 2024
**Version**: 0.1.0

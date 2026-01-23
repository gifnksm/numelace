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
│   ├── sudoku-solver/        # Solving algorithms
│   ├── sudoku-generator/     # Puzzle generation
│   ├── sudoku-game/          # Game logic and state management (planned)
│   └── sudoku-app/           # GUI application (desktop + web) (planned)
└── docs/
    ├── ARCHITECTURE.md       # This file
    └── TESTING.md            # Testing guidelines
```

## Crate Descriptions

### sudoku-core

**Status**: Core data structures implemented ✅

**Purpose**: Fundamental data structures and types for representing Sudoku puzzles.

**Key Components**: `Digit`, `Position`, `CandidateGrid`, `DigitGrid`, generic containers

**Dependencies**: None

**Design**: Semantics Pattern (type-safe indexing), Two-Grid Architecture (separate grids for solving vs I/O)

See [sudoku-core documentation](../crates/sudoku-core/src/lib.rs) for detailed documentation.

---

### sudoku-solver

**Status**: Solver framework implemented ⚙️ (techniques: minimal)

**Purpose**: Solving algorithms using technique-based approach with backtracking fallback.

**Key Components**: `TechniqueSolver`, `BacktrackSolver`, extensible technique system

**Current Techniques**: Naked/Hidden Single (minimal set)

**TODO**: Naked/Hidden Pairs, Pointing Pairs, Box/Line Reduction, X-Wing, etc.

**Dependencies**: `sudoku-core`

**Design**: Two-layer architecture (TechniqueSolver for technique-only solving, BacktrackSolver with backtracking fallback)

See [sudoku-solver documentation](../crates/sudoku-solver/src/lib.rs) for detailed documentation.

---

### sudoku-generator

**Status**: Puzzle generation implemented ✅

**Purpose**: Generates valid Sudoku puzzles with unique solutions.

**Key Components**: `PuzzleGenerator`, `GeneratedPuzzle`, `PuzzleSeed`

**Dependencies**: `sudoku-core`, `sudoku-solver`, `rand`, `rand_pcg`

**Design**: Removal method (generate complete solution, then remove cells with verification)

See [sudoku-generator documentation](../crates/sudoku-generator/src/lib.rs) for detailed documentation.

---

### sudoku-game

**Status**: Game logic minimally implemented ⚙️

**Purpose**: Manages game state, user interactions, and game logic.

**Key Components**: `Game`, `CellState`, `GameError`

**Dependencies**: `sudoku-core`, `sudoku-generator`

**Design**: Permissive validation (allows rule violations), type-safe cell states, accepts any valid solution

**Future Enhancements**: Candidate marks, undo/redo, hints, mistake detection, save/load, timer, statistics

See [sudoku-game documentation](../crates/sudoku-game/src/lib.rs) for detailed documentation.

---

### sudoku-app

**Status**: Planned 📋

**Purpose**: GUI application for both desktop and web platforms using egui/eframe.

**Dependencies**: `sudoku-game`, `eframe`

---

## Architectural Principles

### Crate Separation

**Decision**: `sudoku-core` provides pure data structures only; no solving logic.

**Separation**:

- **Core provides**: Type definitions (`Digit`, `Position`), data structures (`CandidateGrid`, `DigitGrid`), low-level operations (`place()`, `remove_candidate()`), state validation (`is_consistent()`)
- **Core does NOT provide**: Solving techniques (naked singles, hidden singles), search algorithms (backtracking), puzzle generation

**Design Principle**: "Core provides mechanisms, Solver provides policies"

- **Mechanism** (Core): How to place a digit at a specific cell
- **Policy** (Solver): When to place (e.g., when only one candidate remains)

**Benefits**:

- **Reusability**: Core can be used by different solver strategies (technique-based, backtracking, SAT, etc.)
- **Testability**: Core operations tested independently of solving logic
- **Maintainability**: Add new techniques without touching core
- **Extensibility**: Supports sudoku variants (Killer, Irregular, X-sudoku) by composing core primitives

**Trade-offs**: More crates to maintain, but clear separation of concerns and flexibility.

---

### Dependency Management

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

## References

- [Testing Guidelines](TESTING.md) - Project testing philosophy and best practices
- [Rust Book](https://doc.rust-lang.org/book/)
- [egui Documentation](https://docs.rs/egui/)
- [Sudoku Solving Techniques](http://www.sudokuwiki.org/sudoku.htm)
- [BitBoard Techniques](https://www.chessprogramming.org/Bitboards)

---

**Last Updated**: 2026-01-22
**Version**: 0.1.0

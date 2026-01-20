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

- **Basic Types**: `Digit`, `Position` - Type-safe representations of sudoku elements
- **CandidateGrid**: Candidate tracking optimized for solving algorithms
- **DigitGrid**: Simple, intuitive cell-centric interface with string parsing/formatting
- **Generic Containers**: Efficient bitset and array implementations (`BitSet9`, `BitSet81`, `Array9`, `Array81`) with type-safe indexing

**Dependencies**: None

**Design Decisions**:

- **Two-Grid Architecture**: Separation of concerns between solving and simple data access
  - `CandidateGrid`: Digit-centric interface optimized for constraint propagation and solving algorithms
  - `DigitGrid`: Cell-centric interface for intuitive "what's in this cell?" queries
    - Uses `Array81<Option<Digit>, PositionSemantics>` for type-safe cell storage
    - Supports string parsing (`FromStr`) and formatting (`Display`)
    - Provides conversion to/from `CandidateGrid`
  - Allows each type to provide the most natural interface for its use case

- **Type Safety via Semantics**: Generic containers prevent mixing incompatible index types at compile time

For implementation details, see the [crate documentation](../crates/sudoku-core/src/lib.rs).

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

### Two-Grid Architecture

**Decision**: Use separate types for solving (`CandidateGrid`) and data exchange (`DigitGrid`).

**Rationale**:

- Solving algorithms need fast candidate tracking and constraint propagation (digit-centric view)
- Simple data access needs intuitive "what's in this cell?" interface (cell-centric view)
- Each grid type can be optimized for its specific use case without compromise
- Clean separation prevents mixing solving logic with I/O concerns
- `DigitGrid` provides human-friendly string parsing/formatting for puzzle I/O

**Trade-offs**:

- Requires conversion between grid types (via `From`/`Into` traits)
- Two types to maintain instead of one
- Benefits: Better performance, cleaner API, easier to understand and test, natural interfaces for each use case

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

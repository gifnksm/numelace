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
    └── ARCHITECTURE.md       # This file
```

## Crate Descriptions

### sudoku-core

**Status**: Completed ✅

**Purpose**: Provides fundamental data structures and types for representing Sudoku puzzles.

**Key Components**:

- **Basic Types**: `Digit`, `Position` - Type-safe representations of sudoku elements
- **CandidateGrid**: Candidate tracking optimized for solving algorithms
- **DigitGrid**: Simple, intuitive cell-centric interface with string parsing/formatting
- **Generic Containers**: Efficient bitset and array implementations (`BitSet9`, `BitSet81`, `Array9`, `Array81`) with type-safe indexing

**Dependencies**: None

**Design Decisions**:

- **Semantics Pattern**: Type-safe indexing via generic containers
  - See [Semantics Pattern](#semantics-pattern-type-safe-indexing) section below for details
  - Zero runtime cost with compile-time type safety

- **Two-Grid Architecture**: Separation of concerns between solving and simple data access
  - See [Two-Grid Architecture](#two-grid-architecture) section below for detailed analysis
  - `CandidateGrid`: Digit-centric interface optimized for solving algorithms
  - `DigitGrid`: Cell-centric interface for intuitive data access

- **Pure Data Structure Philosophy**: Core provides data structures only, no solving logic
  - See [Core vs Solver Responsibilities](#core-vs-solver-responsibilities) section below

For implementation details, see the [crate documentation](../crates/sudoku-core/src/lib.rs).

---

### sudoku-solver

**Status**: Completed ✅

**Purpose**: Implements solving algorithms using technique-based approach with backtracking fallback.

**Key Components**:

- **`TechniqueSolver`**: Applies human-like solving techniques without backtracking
  - Step-by-step solving with progress tracking
  - Statistics collection (technique usage, step count)
  - Returns when stuck for difficulty evaluation
  
- **`BacktrackSolver`**: Combines techniques with backtracking
  - Uses `TechniqueSolver` first, backtracks when stuck
  - Enumerates all solutions (for puzzle validation)
  - Tracks assumptions and backtrack count

- **Extensible Technique System**: New solving techniques can be added by implementing the `Technique` trait

- **Progress Strategy**: Resets to first technique on any progress (cell placement or candidate removal)

**Dependencies**: `sudoku-core`

**Design Decisions**:

- **Two-layer architecture**: Separates technique-only solving from backtracking
  - Allows difficulty evaluation based on which techniques are needed
  - Useful for puzzle generation with specific technique requirements
  
- **Stateless techniques**: Each technique holds no state, only implements `apply()` method
  
- **Solution enumeration**: `BacktrackSolver::solve()` returns an iterator for finding multiple solutions

For detailed API documentation, see the [crate documentation](../crates/sudoku-solver/src/lib.rs).

---

### sudoku-generator

**Status**: Completed ✅

**Purpose**: Generates valid Sudoku puzzles with unique solutions.

**Key Components**:

- **`PuzzleGenerator`**: Main generator using removal method
  - Two-phase generation: complete solution → cell removal
  - Reproducible generation via `PuzzleSeed` ([u8; 32])
  - Uses `TechniqueSolver` for solvability verification

- **`GeneratedPuzzle`**: Contains problem, solution, and seed
  - Problem: puzzle with cells removed (to be solved)
  - Solution: complete valid grid
  - Seed: allows exact puzzle reproduction

- **Generation Algorithm**:
  1. Generate complete solution using hybrid approach (random first row + backtracking with solver assistance)
  2. Remove cells in shuffled order, verifying solvability after each removal
  3. Keep maximum removed cells while maintaining unique solution

**Dependencies**: `sudoku-core`, `sudoku-solver`, `rand`, `rand_pcg`

**Design Decisions**:

- **Removal Method**: Generates complete grid first, then removes cells
  - Simpler than construction methods
  - Leverages existing `sudoku-solver` infrastructure
  - Naturally produces valid puzzles with unique solutions

- **TechniqueSolver Verification**: Critical design property
  - `TechniqueSolver` uses only logical deduction
  - If puzzle has multiple solutions, solver gets stuck (ambiguous cells)
  - Successful solve proves unique solution
  - This provides both uniqueness guarantee AND difficulty control (human-solvable)

- **Hybrid Solution Generation**: Optimized for performance
  - Step 1: Fill first row with shuffled digits
  - Step 2: Fill remaining cells in top-left box
  - Step 3: Backtracking with MRV heuristic and solver assistance
  - Solver fills logically determined cells, reducing backtracking

- **Type Safety**: `PuzzleSeed` wrapper type instead of raw `[u8; 32]`
  - Prevents seed/data confusion
  - Provides hex `Display` implementation
  - Implements `From<[u8; 32]>` and `Distribution` traits

**Testing**: Property-based tests (proptest) + comprehensive unit tests

- Verifies solvability, reproducibility, and solution subset properties
- 11 unit tests + 3 property tests + 6 doctests

For detailed API documentation, see the [crate documentation](../crates/sudoku-generator/src/lib.rs).

---

### sudoku-game

**Status**: Planned 📋

**Purpose**: Manages game state, user interactions, and game logic.

**Dependencies**: `sudoku-core`, `sudoku-solver`, `sudoku-generator`

---

### sudoku-app

**Status**: Planned 📋

**Purpose**: GUI application for both desktop and web platforms using egui/eframe.

**Dependencies**: `sudoku-game`, `eframe`

---

## Dependency Graph

```text
sudoku-core
    ↓
sudoku-solver
    ↓
sudoku-generator (completed)
    ↓
sudoku-game (planned)
    ↓
sudoku-app (desktop + web) (planned)
```

**Principles**:

- Dependencies flow in one direction (no circular dependencies)
- Lower-level crates have no knowledge of higher-level crates
- Core data structures are independent and reusable
- UI implementations depend on game logic, not vice versa

---

## Key Design Decisions

### Semantics Pattern: Type-Safe Indexing

**Decision**: Use generic containers parameterized by "semantics types" that define index/element mappings.

**Problem**: In sudoku code, digits (1-9), positions (x,y), and cell indices (0-8) are all integers. Using raw arrays allows accidental misuse (e.g., indexing a digit array with a position).

**Solution**: Generic containers (`Array9`, `Array81`, `BitSet9`, `BitSet81`) parameterized by semantics:

- `BitSet9<DigitSemantics>` can only contain `Digit` elements
- `Array81<T, PositionSemantics>` can only be indexed by `Position`
- Type aliases for convenience: `DigitSet`, `DigitPositions`, `HouseMask`

**Benefits**:

- Compile-time type safety prevents index confusion
- Generic implementations shared across all semantics (no code duplication)
- Self-documenting code (type signature reveals purpose)
- Zero runtime cost (PhantomData, inlined arithmetic)

**Trade-offs**: More complex type signatures, but eliminates entire classes of bugs.

For complete documentation, see the [Semantics Pattern section in sudoku-core](../crates/sudoku-core/src/lib.rs#L87-L250).

---

### Two-Grid Architecture

**Decision**: Use separate types for solving (`CandidateGrid`) and data exchange (`DigitGrid`).

**Problem**: Sudoku has two fundamentally different access patterns:

- **Solving**: "Where can digit X go?" (digit-centric, needs fast candidate tracking)
- **Display/I/O**: "What's in cell (x,y)?" (cell-centric, needs simple access)

A single data structure optimized for one pattern performs poorly on the other.

**Solution**:

- **`CandidateGrid`**: Digit-centric representation (`digit_positions[D5]` = bitset of possible positions)
  - O(1) digit queries, efficient candidate tracking via bitwise operations
  - Optimized for solving techniques (hidden singles, naked pairs, etc.)
- **`DigitGrid`**: Cell-centric representation (`cells[Position]` = `Option<Digit>`)
  - O(1) cell access, natural string parsing/formatting
  - Optimized for I/O and display

**Conversion**: One-way `DigitGrid` → `CandidateGrid` via `From` trait. Reverse direction uses explicit methods (lossy conversion).

**Benefits**:

- Each type optimized for its access pattern (performance)
- Clear separation of concerns (solving vs I/O)
- Each type testable independently

**Trade-offs**: Two types to maintain, but no compromise on performance or API clarity.

---

### Core vs Solver Responsibilities

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

### Technique-Based Solving Architecture

**Decision**: Each solving technique is implemented as a separate module.

**Rationale**:

- Easy to add new techniques without modifying existing code
- Clear testing boundaries
- Difficulty evaluation based on technique complexity
- Hint system can explain which technique to use

---

## Testing Strategy

- Each crate has comprehensive unit tests
- Property-based testing for core data structures (proptest)
- Edge case coverage for all public APIs

### Removal Method for Puzzle Generation

**Decision**: Use removal method (generate complete solution, then remove cells) instead of construction methods.

**Rationale**:

- Simpler to implement and understand
- Leverages existing `TechniqueSolver` for verification
- Naturally produces valid puzzles with guaranteed unique solutions
- `TechniqueSolver`'s property (stuck on ambiguous puzzles) provides both uniqueness and difficulty control

**Trade-offs**:

- May not produce puzzles with specific difficulty levels (depends on available techniques)
- Cannot control symmetry or aesthetic patterns in initial implementation
- Benefits: Simple, reliable, produces human-solvable puzzles, builds on existing infrastructure

---

## References

- [Rust Book](https://doc.rust-lang.org/book/)
- [egui Documentation](https://docs.rs/egui/)
- [Sudoku Solving Techniques](http://www.sudokuwiki.org/sudoku.htm)
- [BitBoard Techniques](https://www.chessprogramming.org/Bitboards)

---

**Last Updated**: 2026-01-22
**Version**: 0.1.0

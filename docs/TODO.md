# TODO

This file tracks tasks that must be done to achieve the project goals.

**Workflow**: For initial component development (applies to sudoku-generator, sudoku-game, sudoku-app):

1. **Planning Phase**:
   - Create a design document at `docs/design/COMPONENT.md`
   - Based on the design, add specific implementation tasks to this TODO
   - Commit: `docs: add design for COMPONENT`

2. **Implementation Phase**:
   - Implement code and tests
   - Keep focus on code (do not update TODO.md during implementation)
   - Commit as needed: `feat(COMPONENT): implement FEATURE` (multiple commits OK)

3. **Completion Phase** (after all tests pass):
   - Mark all tasks as completed in this TODO
   - Delete the design document
   - Preserve essential design decisions in crate documentation and ARCHITECTURE.md
   - Update status in README.md (Current Status section)
   - Update status in ARCHITECTURE.md (Crate Descriptions section and status markers)
   - Commit all documentation updates together: `docs: complete COMPONENT and update project documentation`

4. **Next Component** (optional):
   - Remove completed section from this TODO to keep it clean
   - Commit: `docs: archive completed COMPONENT tasks`

**Note**: Once all core components are implemented, this TODO will transition to tracking features, improvements, and bugs rather than component-by-component development. The workflow may be adapted at that time.

---

## sudoku-generator: Puzzle Generation

- [x] Create design document at `docs/design/sudoku-generator.md`
  - Consider aspects such as: generation algorithm, API design, difficulty evaluation, etc.
- [x] Add specific implementation tasks to this TODO based on design decisions
  - [x] Create `crates/sudoku-generator` crate
  - [x] Implement `PuzzleGenerator` struct with `TechniqueSolver` dependency
  - [x] Implement complete grid generation using random placement + backtracking
  - [x] Implement cell removal algorithm with shuffled positions
  - [x] Implement solvability verification using `TechniqueSolver`
  - [x] Add `rand` and `rand_pcg` dependencies
  - [x] Write unit tests for generation logic
  - [x] Write property-based tests using `proptest`
  - [x] Update workspace `Cargo.toml` to include new crate
- [x] On completion:
  - [x] Delete design document
  - [x] Preserve essential design decisions in crate documentation and ARCHITECTURE.md
  - [x] Update README.md status (Current Status section)
  - [x] Update ARCHITECTURE.md status (Crate Descriptions section)
  - [x] Mark all tasks as completed in this TODO

**Status**: ✅ Completed. Design decisions preserved in ARCHITECTURE.md and crate documentation.

---

## sudoku-game: Game Logic

- [ ] Create design document at `docs/design/sudoku-game.md`
  - Consider aspects such as: game state structure, operation APIs, undo/redo mechanism, save/load format, interaction with other components, etc.
- [ ] Add specific implementation tasks to this TODO based on design decisions
- [ ] On completion:
  - [ ] Delete design document
  - [ ] Preserve essential design decisions in crate documentation and ARCHITECTURE.md
  - [ ] Update README.md status (Current Status section)
  - [ ] Update ARCHITECTURE.md status (Crate Descriptions section)
  - [ ] Mark all tasks as completed in this TODO

**Note**: This is marked as "Planned" in ARCHITECTURE.md and README.md

---

## sudoku-app: GUI Implementation

- [ ] Create design document at `docs/design/sudoku-app.md`
  - Consider aspects such as: UI layout, user interaction flow, egui/eframe integration, desktop/WASM build configuration, state management, etc.
- [ ] Add specific implementation tasks to this TODO based on design decisions
- [ ] On completion:
  - [ ] Delete design document
  - [ ] Preserve essential design decisions in crate documentation and ARCHITECTURE.md
  - [ ] Update README.md status (Current Status section)
  - [ ] Update ARCHITECTURE.md status (Crate Descriptions section)
  - [ ] Mark all tasks as completed in this TODO

**Note**: This is marked as "Planned" in ARCHITECTURE.md and README.md. Desktop GUI support using egui/eframe is explicitly mentioned in project goals.

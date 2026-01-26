# TODO

This file tracks tasks that must be done to achieve the project goals.

**Workflow**: For initial component development (applies to numelace-generator, numelace-game, numelace-app):

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
   - Preserve essential design decisions in crate documentation and ARCHITECTURE.md
   - Move future enhancement ideas from design document to ARCHITECTURE.md
   - Delete the design document
   - Update status in README.md (Current Status section)
   - Update status in ARCHITECTURE.md (Crate Descriptions section and status markers)
   - Commit all documentation updates together: `docs: complete COMPONENT and update project documentation`

4. **Next Component** (optional):
   - Remove completed section from this TODO to keep it clean
   - Commit: `docs: archive completed COMPONENT tasks`

**Note**: Once all core components are implemented, this TODO will transition to tracking features, improvements, and bugs rather than component-by-component development. The workflow may be adapted at that time.

---

## numelace-app: GUI Implementation

- [x] Create design document at `docs/design/numelace-app.md`
  - Consider aspects such as: UI layout, user interaction flow, egui/eframe integration, desktop/WASM build configuration, state management, etc.
- [x] Add specific implementation tasks to this TODO based on design decisions
  - [x] Update `crates/numelace-app/Cargo.toml` dependencies to include `numelace-game`, `numelace-generator`, and `numelace-solver`
  - [x] Add app state: `Game`, selected cell, and optional status message
  - [x] Implement new game creation (create solver, generator, puzzle, and `Game`)
  - [x] Render 9x9 board with clear 3x3 boundaries
  - [x] Implement cell selection (mouse click)
  - [x] Implement keyboard input: digits 1-9, Backspace/Delete, arrow navigation, optional `Esc` to clear selection
  - [x] Show status text for in-progress vs solved state
  - [x] Add window sizing: initial size, resizable, and minimum size constraints
  - [x] Extract input handling into small methods for testability (optional)
- [x] On completion:
  - [x] Preserve essential design decisions in crate documentation and ARCHITECTURE.md
  - [x] Move future enhancement ideas to ARCHITECTURE.md
  - [x] Delete design document
  - [x] Update README.md status (Current Status section)
  - [x] Update ARCHITECTURE.md status (Crate Descriptions section)
  - [x] Mark all tasks as completed in this TODO

**Note**: Desktop GUI support using egui/eframe is explicitly mentioned in project goals.

---

## numelace-solver: Technique Extensions

- [ ] Identify and prioritize additional solving techniques to implement
- [ ] Implement new techniques with tests and documentation
- [ ] Update ARCHITECTURE.md with technique descriptions

**Note**: This is an enhancement task to expand solver capabilities. Not blocking core application development.

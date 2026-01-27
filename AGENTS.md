# AI Agent Guidance

## Read First

Before proposing changes or plans, read these files in order:

1. `docs/BACKLOG.md`
2. `docs/DESIGN_LOG.md`
3. `docs/WORKFLOW.md`
4. `docs/ARCHITECTURE.md`
5. `docs/TESTING.md`
6. `docs/DOCUMENTATION_GUIDE.md`

## Working Style

- Keep suggestions lightweight and aligned with the current backlog.
- If a new decision is made, append a short entry to `docs/DESIGN_LOG.md`.
- If new work is requested or discovered, add it to `docs/BACKLOG.md`.
- Prefer minimal edits and preserve the existing project structure and conventions.

## Documentation Updates

- Long-lived architectural decisions belong in `docs/ARCHITECTURE.md`.
- Short-term decisions and rationale belong in `docs/DESIGN_LOG.md`.
- Follow `docs/TESTING.md` for test scope and `docs/DOCUMENTATION_GUIDE.md` for doc conventions.
- Use `Numelace` for the app/brand name and `Sudoku` when referring to the puzzle rules.

## Defaults

- Don’t introduce scheduling or strict plans unless explicitly requested.
- Ask clarifying questions when scope or intent is ambiguous.
- Seek confirmation before large changes (e.g., deletions, restructures, automated fixes, or commits).
- Confirm who will implement code changes (user or agent) before starting.
- When committing, confirm before running the commit; choose an appropriate message (prefix optional) and add a brief body only when intent, impact, or caveats aren't captured; e.g., `docs: clarify commit confirmation rule`.

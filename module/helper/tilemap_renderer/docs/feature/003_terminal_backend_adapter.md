# Feature: Terminal Backend Adapter

`adapters::TerminalBackend` is the planned `Backend` implementation for ASCII/Unicode terminal output, behind the `adapter-terminal` feature.

### Scope

- **Purpose**: Let a command stream produce a terminal-rendered (ASCII/Unicode) preview.
- **Responsibility**: Record the adapter's actual (stub) implementation state, distinct from its originally intended design.
- **In Scope**: The current state of `src/adapters/terminal.rs` and the feature gate it sits behind.
- **Out of Scope**: SVG and WebGL2 adapters (see [feature/001_svg_backend_adapter.md](001_svg_backend_adapter.md), [feature/002_webgl2_backend_adapter.md](002_webgl2_backend_adapter.md)); any rendering behavior, since none is implemented yet.

### Design

**Current state — stub only.** `src/adapters/terminal.rs` is a 7-line file: a module doc comment stating "Status: stub only — implementation deferred to a follow-up PR", an empty `mod private {}`, and an empty `mod_interface::mod_interface! {}` block. The `adapter-terminal` feature gate compiles, but no `Backend` implementation, rendering logic, or type exists in the module yet. This matches `roadmap.md`, which lists the terminal adapter under "deferred to follow-up PRs" with the same "stub only" characterization.

**Intended design** (not yet implemented): ASCII/Unicode rendering via Bresenham line drawing, ANSI color support, and configurable output dimensions. This description is carried forward here as the documented *intent* for the eventual implementation — it must not be read as a description of current behavior. Any functional-requirement tracking for this adapter (sprite/mesh/batch support, effects, gradient approximation) is future scope, listed in `roadmap.md`'s "terminal adapter gaps" section, not in this doc entity — per this crate's documentation split, forward-looking scope belongs in `roadmap.md`, not `docs/`.

Status is tracked as deferred (⏸️), matching both the source module's own doc comment and `roadmap.md`.

### Patterns

| File | Relationship |
|------|--------------|
| [pattern/001_ports_and_adapters_backend_architecture.md](../pattern/001_ports_and_adapters_backend_architecture.md) | Intended to become a third `Backend` implementation within the crate's hexagonal architecture |

### Sources

| File | Relationship |
|------|--------------|
| `src/adapters/terminal.rs` | Stub module — no implementation |

### Tests

| File | Relationship |
|------|--------------|
| — | No implementation exists yet, so no tests exist |

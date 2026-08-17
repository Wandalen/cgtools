# Feature: None Backend Adapter

`adapters::NoneBackend` implements the core `Backend` trait as a complete no-op, behind the `adapter-none` feature — first-class support for driving a command stream with no GPU or document work at all.

### Scope

- **Purpose**: Let a command stream run to completion with zero rendering work, for math-only simulation (physics, layout, or command-stream authoring/testing) where output is never consumed.
- **Responsibility**: Cross-reference the no-op adapter's source and its deliberately trivial contract — accepts everything, produces `Output::Presented`, inspects nothing.
- **In Scope**: `src/adapters/none.rs` and its full `Backend` implementation.
- **Out of Scope**: Every other adapter (see [001](001_svg_backend_adapter.md), [002](002_webgl2_backend_adapter.md), [003](003_terminal_backend_adapter.md), [005](005_webgpu_backend_adapter.md), [006](006_native_backend_adapter.md)); the "math-only simulation" use case's own call sites, which live outside this crate.

### Design

`NoneBackend` is a complete, working implementation, not a stub — unlike the terminal adapter's deferred status ([003](003_terminal_backend_adapter.md)), there is no follow-up PR pending here; the no-op behavior *is* the finished feature. `assets_load` and `submit` both unconditionally return `Ok(())` without inspecting their input, `output()` always returns `Output::Presented`, `resize()` is a no-op, and `capabilities()` returns `Capabilities::default()` (every flag `false`, matching a backend that renders nothing).

This is deliberately distinct from an unimplemented command family returning `RenderError::Unsupported` elsewhere in the crate — `NoneBackend` never errors on any input, because accepting and discarding everything is the entire contract, not a partial one. The adapter exists so a caller can drive the same `RenderCommand` stream through a pipeline that only cares about simulation state (e.g. physics or layout math expressed as commands) without requiring a real backend — see `docs/adr/003_d2_stack_hal_adoption.md` Decision #2, which formalizes this as "math-only simulation, no rendering."

Status is tracked as complete (✅) — the smallest possible `Backend` implementation, fully honest about doing nothing.

### Patterns

| File | Relationship |
|------|--------------|
| [pattern/001_ports_and_adapters_backend_architecture.md](../pattern/001_ports_and_adapters_backend_architecture.md) | This adapter is one `Backend` implementation within the crate's hexagonal architecture |

### Sources

| File | Relationship |
|------|--------------|
| `src/adapters/none.rs` | `NoneBackend` — the complete no-op `Backend` implementation |

### Tests

| File | Relationship |
|------|--------------|
| `tests/none_backend_test.rs` | Contract tests: `assets_load`/`submit` accept non-empty input and never error, `output` always `Presented`, `resize` before and after `submit` never changes `output`, `capabilities()` matches `Capabilities::default()` field-for-field, `submit` ignores a missing-asset sprite reference rather than erroring |

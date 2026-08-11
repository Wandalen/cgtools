# Non Functional Requirement: Minimal Abstraction Overhead

### Scope

- **Purpose**: Keep the safe Rust facade from meaningfully slowing down WebGPU operations relative to raw `web-sys` calls.
- **Responsibility**: Document the crate's performance quality attribute and its intended measurement method.
- **In Scope**: CPU-time overhead and per-frame allocation behavior of `minwebgpu`'s wrapper functions and descriptor builders.
- **Out of Scope**: WebGPU driver- or browser-side GPU performance, which is outside the crate's control.

### Quality Attribute

Performance — abstraction overhead versus raw `web-sys`.

### Statement

`minwebgpu`'s wrapper functions and descriptor builders must introduce negligible CPU-time overhead over equivalent raw `web-sys` calls, and must avoid unnecessary heap allocations during per-frame operations (e.g. inside a render pass).

### Measurement Method

Benchmark a `minwebgpu`-based render loop against an equivalent raw `web-sys` implementation and compare CPU time for the same operation sequence; per-frame allocation counts can be checked by profiling a render-pass call path for heap activity.

### Acceptance Threshold

The pre-migration specification set a target of under 5% CPU-time overhead versus raw `web-sys` calls. No benchmark currently exists in this crate to confirm the current implementation against that target — treat this as the design target, not a verified measurement.

### Features

| File | Relationship |
|------|--------------|
| [feature/003_pipeline_management.md](../feature/003_pipeline_management.md) | Heaviest per-frame use of nested descriptor builders |

### Sources

| File | Relationship |
|------|--------------|
| `src/descriptor/` | Descriptor-builder types whose overhead this bounds |
| `src/state/` | Nested pipeline state builders whose overhead this bounds |

### Tests

No automated tests or benchmarks exist for this crate at the time of this migration.

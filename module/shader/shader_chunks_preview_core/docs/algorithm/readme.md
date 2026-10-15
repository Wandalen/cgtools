# Algorithm Doc Definition

An **algorithm** documents a HOW — a step-by-step computational procedure
with correctness properties worth stating explicitly. In
`shader_chunks_preview_core`, this collection is the navigational hub for
the one algorithmic procedure the crate implements: detecting which shape
(if any) a value chunk's export matches, and how the chosen shape is
rendered. This collection holds one instance per algorithm; the table
below is the index into them.

### Scope

- **Purpose**: Navigational hub for `shader_chunks_preview_core`'s
  step-by-step computational procedures.
- **Responsibility**: Document each algorithm's abstract behavior and
  concrete step-by-step procedure.
- **In Scope**: The value-function shape detection heuristic
  `value_fn_of`/`harness_synthesize` implement.
- **Out of Scope**: The fragment-chunk mode (`//@ stage: fragment`) —
  a separate, unrelated selection path in `bundle_build` that this
  algorithm does not participate in.

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [Value Function Shape Detection](001_value_function_shape_detection.md) | Detect which of `f32`/`vec2f`/`vec3f` a value chunk's export returns, and how each is written to the render target | ✅ |

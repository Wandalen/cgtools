# Invariant: Z-Layer Draw Ordering

Command order in, paint order out: submission order is the one ordering
contract every backend honors; `Transform::depth` is a coarse z-layer index
layered on top of it, honored only where a depth buffer exists.

### Scope

- **Purpose**: State the draw-ordering contract callers may rely on across backends — the d2 stack's ordering invariant.
- **Responsibility**: Pin what is portable (submission order), what is conditional (`Transform::depth`), and where each is enforced.
- **In Scope**: Ordering semantics of the command stream as observed in every `Backend`'s output.
- **Out of Scope**: The WebGL2 depth-buffer implementation itself (see [feature/002_webgl2_backend_adapter.md](../feature/002_webgl2_backend_adapter.md)); axis orientation (see [invariant/001_y_up_coordinate_system.md](001_y_up_coordinate_system.md)); blend-mode correctness per backend (`Capabilities::supported_blend_modes`).

### Invariant Statement

Commands submitted at equal `Transform::depth` — including the default
`depth : 0.0` — composite in **submission order**: a later command paints
over an earlier one. This holds on every backend. On backends with a depth
buffer (currently WebGL2 only), `Transform::depth` additionally reorders
draws coarsely, per field, within `[-RenderConfig::max_depth, max_depth]` —
and is **reliable only for fully opaque draws**. Backends without a depth
buffer (SVG) ignore `depth` entirely; for them submission order is the whole
contract.

### Enforcement Mechanism

- **SVG**: structural — the adapter emits elements in submission order
  (document order is paint order in SVG), and `src/adapters/svg.rs` contains
  no read of `Transform::depth` at all; its only depth-named state is
  `group_depth`, a group-nesting counter.
- **WebGL2**: `DEPTH_TEST` with `LEQUAL` (see `roadmap.md`, WebGL2 adapter
  section). `LEQUAL` passes fragments whose depth *equals* the stored depth,
  so equal-`depth` draws still resolve to submission order — the same
  contract, GPU-enforced. The shader divides `depth` by `u_max_depth` before
  writing clip-space z; values outside `[-max_depth, max_depth]` are clipped
  by the GPU (the draw disappears). For batches the effective depth is
  `parent_depth + instance_depth`, subject to the same range.
- **Type level**: `RenderConfig::max_depth`'s doc comment
  (`src/types.rs`) records the range contract, the precision trade-off
  (larger range → coarser depth precision), and that a zero or negative
  `max_depth` is unsupported.

### Violation Consequences

- A scene relying on `depth` for layering renders correctly on WebGL2 and
  silently *flat* on SVG — layers collapse to submission order. Portable
  callers must submit in back-to-front order and treat `depth` as an
  optimization, not a semantic.
- Translucent draws ordered by `depth` on WebGL2 blend incorrectly (depth
  test discards occluded fragments before blending); translucency must be
  submitted back-to-front regardless of `depth`.
- A `depth` outside `[-max_depth, max_depth]` is clipped — the draw vanishes
  rather than clamping.

### Features

| File | Relationship |
|------|--------------|
| [feature/001_svg_backend_adapter.md](../feature/001_svg_backend_adapter.md) | Backend where submission order is the entire ordering contract |
| [feature/002_webgl2_backend_adapter.md](../feature/002_webgl2_backend_adapter.md) | Backend that additionally honors `Transform::depth` via the depth buffer |

### Sources

| File | Relationship |
|------|--------------|
| `roadmap.md` | WebGL2 adapter section: `LEQUAL`, per-field range, opaque-only reliability, batch depth sum |
| `src/adapters/svg.rs` | Document-order emission; no `Transform::depth` read (only the `group_depth` nesting counter) |
| `src/adapters/webgl.rs` | Depth-honoring implementation |
| `src/types.rs` | `Transform::depth` (default `0.0`), `RenderConfig::max_depth` and its range/precision doc |

### Tests

| File | Relationship |
|------|--------------|
| `tests/types_test.rs` | Pins `Transform`/`RenderConfig` defaults the contract builds on |
| — | No dedicated cross-backend ordering test yet; SVG document-order output is exercised implicitly by the SVG adapter suite |

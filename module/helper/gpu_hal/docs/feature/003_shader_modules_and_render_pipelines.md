# Feature: Shader Modules & Render Pipelines

### Scope

- **Purpose**: Compile shader source into a `ShaderModule` and assemble the fixed-function and programmable pipeline state into a `RenderPipeline`.
- **Responsibility**: Document the shader-compilation and pipeline-creation API's design.
- **In Scope**: `shader_module_create`, `ShaderSource`, `render_pipeline_create`, `RenderPipelineDesc`, `VertexBufferLayout`/`VertexAttribute`/`VertexFormat`, `DepthState`.
- **Out of Scope**: Bind group layouts consumed by `RenderPipelineDesc::bind_group_layouts` (see `feature/004`); recording draws through the created pipeline (see `feature/005`).

### Design

`ShaderSource<'a>` carries canonical WGSL (`wgsl: &'a str`) plus an optional per-backend GLSL override (`glsl_vertex`/`glsl_fragment: Option<&'a str>`) — the shader-access contract of ADR-001 §5 in concrete form. `shader_module_create` dispatches on the active backend: WebGPU and native both consume `source.wgsl` directly and, per their own doc comment, "never fail this call"; WebGL requires **both** GLSL override slots to be `Some` and returns `Err(Error::Unsupported("the WebGL backend requires both GLSL override slots of ShaderSource"))` otherwise — there is no WGSL-to-GLSL transpilation in the crate today, so a caller building a WebGL-inclusive binary that only ever supplies `wgsl` will get an `Unsupported` at the first WebGL call, not a compile error, since the same source compiles fine for the other two backends.

`render_pipeline_create(&RenderPipelineDesc)` takes a shader, vertex/fragment entry point names, a `VertexBufferLayout` slice (`stride` plus a `Vec<VertexAttribute>` of `{ location, format: VertexFormat, offset }`), the bind group layouts the pipeline's shader references, the target `color_format`, an optional `DepthState` (format plus the v0 fixed function set: depth test `less`, depth write on), and a `cull_back` flag. It fails with `Error::WebGpu` on a WebGPU pipeline-creation failure, or `Error::WebGl` if the vertex/fragment GLSL pair fails to compile and link — native never fails this call.

### Invariants

| File | Relationship |
|------|--------------|
| [invariant/001_result_based_error_handling_scoped_panics.md](../invariant/001_result_based_error_handling_scoped_panics.md) | Both constructors return `Result<_, Error>`, including the WebGL missing-GLSL-slot case |

### Patterns

| File | Relationship |
|------|--------------|
| [pattern/001_enum_per_backend_dispatch_one_step_drilldown.md](../pattern/001_enum_per_backend_dispatch_one_step_drilldown.md) | `ShaderModule`/`RenderPipeline` are backend-tagged enums like every other handle |
| [../../../../../docs/pattern/002_strict_layering_one_step_drilldown.md](../../../../../docs/pattern/002_strict_layering_one_step_drilldown.md) | Named `ShaderSource` before this crate existed: "the future HAL carries canonical WGSL plus a per-backend override slot for the same reason" |

### Sources

| File | Relationship |
|------|--------------|
| `src/device.rs` | `shader_module_create`, `render_pipeline_create` |
| `src/resource.rs` | `ShaderModule`/`RenderPipeline` enums |
| `src/types.rs` | `ShaderSource`, `DepthState`, `VertexAttribute`, `VertexBufferLayout`, `VertexFormat` |

### Tests

| File | Relationship |
|------|--------------|
| `tests/native_backend_test.rs` | `shader_module_create` (WGSL-only, native arm) and `render_pipeline_create` exercised in both `triangle_render_readback` and `texture_write_readback` |

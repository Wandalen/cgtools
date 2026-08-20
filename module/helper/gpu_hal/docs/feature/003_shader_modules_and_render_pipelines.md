# Feature: Shader Modules & Render Pipelines

### Scope

- **Purpose**: Compile shader source into a `ShaderModule` and assemble the fixed-function and programmable pipeline state into a `RenderPipeline`.
- **Responsibility**: Document the shader-compilation and pipeline-creation API's design.
- **In Scope**: `shader_module_create`, `ShaderSource`, `render_pipeline_create`, `RenderPipelineDesc`, `VertexBufferLayout`/`VertexAttribute`/`VertexFormat`, `DepthState`.
- **Out of Scope**: Bind group layouts consumed by `RenderPipelineDesc::bind_group_layouts` (see `feature/004`); recording draws through the created pipeline (see `feature/005`).

### Design

`ShaderSource<'a>` carries canonical WGSL (`wgsl: &'a str`) plus an optional per-backend GLSL override (`glsl_vertex`/`glsl_fragment: Option<&'a str>`) — the shader-access contract of ADR-001 §5 in concrete form. `shader_module_create` dispatches on the active backend: WebGPU and native both consume `source.wgsl` directly and, per their own doc comment, "never fail this call"; WebGL requires **both** GLSL override slots to be `Some` and returns `Err(Error::Unsupported("the WebGL backend requires both GLSL override slots of ShaderSource"))` otherwise; Vulkan also consumes `source.wgsl` directly, but compiles it to SPIR-V via `naga` first, so unlike WebGPU and native it can fail — with `Error::Vulkan` if that compilation or the underlying `vkCreateShaderModule` call fails — `shader_module_create`'s WebGL arm performs no transpilation itself — it only checks that both GLSL slots are already populated — so a caller building a WebGL-inclusive binary that only ever supplies `wgsl` will get an `Unsupported` at the first WebGL call, not a compile error, since the same source compiles fine for the other three backends. The crate does provide WGSL-to-GLSL transpilation, just not inside this runtime dispatch: `webgl_build::wgsl_to_webgl_glsl()` (`src/webgl_build.rs`, `webgl-glsl-build` feature) translates WGSL to GLSL ES 300 via `naga`, meant to be called from a downstream crate's own `build.rs` to populate the override slots ahead of time — see `renderer/build.rs`, which uses it as a required build-dependency instead of hand-porting the GLSL.

`render_pipeline_create(&RenderPipelineDesc)` takes a shader, vertex/fragment entry point names, a `VertexBufferLayout` slice (`stride` plus a `Vec<VertexAttribute>` of `{ location, format: VertexFormat, offset }`), the bind group layouts the pipeline's shader references, the target `color_format`, an optional `DepthState` (format plus the v0 fixed function set: depth test `less`, depth write on), and a `cull_back` flag. It fails with `Error::WebGpu` on a WebGPU pipeline-creation failure, or `Error::WebGl` if the vertex/fragment GLSL pair fails to compile and link, or `Error::Vulkan` if an entry point name contains an interior nul byte or the underlying pipeline layout, render pass, or graphics pipeline creation fails — native never fails this call.

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
| `src/vulkan.rs` | Vulkan arms: `shader_module_create` (WGSL→SPIR-V via `naga`), `render_pipeline_create` |

### Tests

| File | Relationship |
|------|--------------|
| `tests/native_backend_test.rs` | `shader_module_create` (WGSL-only, native arm) and `render_pipeline_create` exercised in both `triangle_render_readback` and `texture_write_readback` |
| `tests/vulkan_backend_test.rs` | `triangle_render_readback` exercises both `shader_module_create` and `render_pipeline_create` |

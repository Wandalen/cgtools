# Feature: WebGL2 Backend Adapter

`adapters::WebGlBackend` implements the core `Backend` trait for hardware-accelerated WebGL2 rendering on the `wasm32` target, behind the `adapter-webgl` feature.

### Scope

- **Purpose**: Let a command stream drive real-time, GPU-accelerated rendering in a browser.
- **Responsibility**: Cross-reference the WebGL2 adapter's source, the pitfalls confirmed in its GPU buffer handling, and its actual (partial) capability status.
- **In Scope**: Sprite, mesh, and instanced-batch rendering; async and sync image loading; per-instance depth and blend-mode handling.
- **Out of Scope**: SVG and Terminal adapters (see [feature/001_svg_backend_adapter.md](001_svg_backend_adapter.md), [feature/003_terminal_backend_adapter.md](003_terminal_backend_adapter.md)); the Y-up invariant itself (see [invariant/001](../invariant/001_y_up_coordinate_system.md), satisfied natively here with no adapter-side logic).

### Design

The implementation is split across two files to stay under the workspace's per-file size budget: `adapters/webgl.rs` holds `WebGlBackend`, the sprite/mesh renderers, and the async image loader; `adapters/webgl/webgl_helpers.rs` (a `mod_interface::layer` submodule) holds self-contained helpers — the GPU-side array type, instance-data layouts, GPU resource handles, VAO setup, the async `Loadable` mechanism, and GL enum-mapping helpers.

Instanced sprite and mesh batches are backed by `ArrayBuffer<T>`, a GPU-side growable array (`ARRAY_BUFFER`) that doubles capacity via `copy_buffer_sub_data` (GPU-to-GPU, no CPU readback) and removes elements via swap-remove. The swap-remove step needs a persistent scratch buffer to stay spec-compliant — see [pitfall/001](../pitfall/001_arraybuffer_swap_remove_buffer_binding_violation.md). Per-instance GPU data is fixed-layout (`SpriteInstanceData` 72 bytes, `MeshInstanceData` 56 bytes, both including a per-instance tint and depth), and the byte offsets the VAO attribute setup reads from those structs are hardcoded rather than computed — see [pitfall/002](../pitfall/002_gpu_instance_struct_field_reorder_desync.md) for exactly what is and isn't guarded against a future field change. VAO attribute bindings are configured once at batch create/unbind time and simply bound (not reconfigured) at each draw call.

Depth ordering uses the WebGL2 depth buffer (`DEPTH_TEST` enabled, `LEQUAL` comparison so identical-depth draws fall back to submission order rather than rejecting the later one). `Transform::depth` is mapped into clip-space `z` by dividing by `RenderConfig::max_depth` (default `1.0`); the valid per-field range is `[-max_depth, max_depth]` and the GPU clips values outside it rather than the adapter validating them. For batch instances, the constraint applies to the **sum** `parent_depth + instance_depth`, not each independently. This ordering is reliable only for fully opaque draws — translucent content must still be submitted back-to-front by the caller, since alpha blending order is not resolved by the depth buffer. `DEPTH_TEST` also depends on the caller having created the WebGL2 context with a depth attachment; the standard `getContext` default (`depth: true`) provides one, but a caller-supplied context created with `depth: false` makes `DEPTH_TEST` a silent no-op — depth ordering then falls back to submission order with no error (confirmed via the depth-buffer doc comment in `src/adapters/webgl.rs`).

Image upload has two paths with different synchronicity: `ImageSource::Bitmap` (already-decoded bytes) uploads synchronously via a direct `tex_image_2d` call; `ImageSource::Path` loads asynchronously through an `HtmlImageElement`, using `Closure::once_into_js` for one-shot `onload`/`onerror` handlers so the browser releases the captured Rust closure (and the `Rc<RefCell<GpuResources>>` it holds) once the event fires — this is what lets dropping a `WebGlBackend` actually release its GPU resources rather than leaking them via a retained closure cycle.

Blend modes `Normal`, `Add`, `Multiply`, and `Screen` map directly to a single `blend_func`/`blend_equation` call each and are hardware-accelerated; `BlendMode::Overlay` (the Photoshop-style conditional blend) cannot be expressed as one `blend_func` call and currently falls back to `Normal` — silently, from the caller's point of view, unless it inspects `capabilities().supported_blend_modes`, which deliberately omits `Overlay` from its four-entry list. `capabilities().blend_modes` is `false` specifically to signal that not every `BlendMode` variant renders correctly on this backend.

Command families the adapter does not yet implement — path (`BeginPath`..`EndPath`), text (`BeginText`..`EndText`), and group (`BeginGroup`..`EndGroup`) commands — are not fully silent: each family logs one `console::warn_1` diagnostic per `BeginPath`/`BeginText`/`BeginGroup` occurrence (referencing the relevant `capabilities()` flag), deliberately not per sub-command — a 1000-segment path still produces one warning, not 1000, since only the opener matches a warning arm and `MoveTo`/`LineTo`/`Char`/etc. are silent no-ops. There is no dedup across multiple openers in one stream or across `submit()` calls: five separate `BeginPath`...`EndPath` runs still produce five warnings. This is more precise than treating the whole family as unconditionally silent: there is a visible (browser-console) signal per path/text-run/group, just not a `RenderError` or a change to the rendered output.

Given the number of unimplemented command families and the `Overlay` blend-mode gap, this adapter's status is tracked as partial (⚠️), matching its characterization in `roadmap.md` ("WebGL2 adapter is partially implemented").

### Invariants

| File | Relationship |
|------|--------------|
| [invariant/001_y_up_coordinate_system.md](../invariant/001_y_up_coordinate_system.md) | Satisfied natively (OpenGL's own convention); no adapter-side conversion |

### Patterns

| File | Relationship |
|------|--------------|
| [pattern/001_ports_and_adapters_backend_architecture.md](../pattern/001_ports_and_adapters_backend_architecture.md) | This adapter is one `Backend` implementation within the crate's hexagonal architecture |

### Pitfalls

| File | Relationship |
|------|--------------|
| [pitfall/001_arraybuffer_swap_remove_buffer_binding_violation.md](../pitfall/001_arraybuffer_swap_remove_buffer_binding_violation.md) | `ArrayBuffer<T>::swap_remove`'s scratch-buffer requirement |
| [pitfall/002_gpu_instance_struct_field_reorder_desync.md](../pitfall/002_gpu_instance_struct_field_reorder_desync.md) | Hardcoded VAO offsets vs. size/align-only compile-time assertions |

### Sources

| File | Relationship |
|------|--------------|
| `src/adapters/webgl.rs` | `WebGlBackend`, sprite/mesh renderers, async image loader, command dispatch |
| `src/adapters/webgl/webgl_helpers.rs` | `ArrayBuffer<T>`, instance-data layouts, GPU resource handles, VAO setup, GL mapping helpers |
| `src/adapters/shaders/sprite.vert`, `sprite.frag` | Single-sprite shader pair |
| `src/adapters/shaders/sprite_batch.vert`, `sprite_batch.frag` | Instanced sprite-batch shader pair |
| `src/adapters/shaders/mesh.vert`, `mesh.frag` | Single-mesh shader pair |
| `src/adapters/shaders/mesh_batch.vert`, `mesh_batch.frag` | Instanced mesh-batch shader pair |

### Tests

| File | Relationship |
|------|--------------|
| — | No automated test currently exercises this adapter — it requires a `wasm32` target and a live WebGL2 context, and no `wasm-bindgen-test` harness exists in this crate yet; core and SVG-adapter tests run under plain `cargo test` and do not cover this file |

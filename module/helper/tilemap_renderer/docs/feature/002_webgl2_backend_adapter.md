# Feature: WebGL2 Backend Adapter

`adapters::WebGlBackend` implements the core `Backend` trait for hardware-accelerated WebGL2 rendering on the `wasm32` target, behind the `adapter-webgl` feature.

### Scope

- **Purpose**: Let a command stream drive real-time, GPU-accelerated rendering in a browser.
- **Responsibility**: Cross-reference the WebGL2 adapter's source, the pitfalls confirmed in its GPU buffer handling, and its actual (partial) capability status.
- **In Scope**: Sprite, mesh, and instanced-batch rendering; async and sync image loading; per-instance depth and blend-mode handling.
- **Out of Scope**: SVG, Terminal, None, WebGPU, and Native adapters (see [001](001_svg_backend_adapter.md), [003](003_terminal_backend_adapter.md), [004](004_none_backend_adapter.md), [005](005_webgpu_backend_adapter.md), [006](006_native_backend_adapter.md)); the Y-up invariant itself (see [invariant/001](../invariant/001_y_up_coordinate_system.md), satisfied natively here with no adapter-side logic).

### Design

The implementation is split across six files, each under the workspace's 1500-line source-file ceiling: `adapters/webgl.rs` holds `WebGlBackend` (command dispatch, asset loading, context-loss handling); four `mod_interface::layer` submodules under `adapters/webgl/` hold the rest, and a fifth child module, `webgl_surfaces.rs`, carries a second `impl WebGlBackend` block for the offscreen render-target lifecycle (create / reset / begin / end / draw), `scissor_set`, `fill` and `image_tiled_draw`. It is a plain `mod` rather than a layer because it adds methods instead of exporting items, and the backend fields it reads are `pub( super )`, so they stay private to `adapters::webgl`. `webgl_renderers.rs` has the sprite/mesh renderers (shader programs and draw calls), `webgl_textures.rs` the texture uploads (sync bitmaps and the async image loader), and `webgl_helpers.rs` self-contained helpers — the GPU-side array type, instance-data layouts, GPU resource handles, VAO setup, the async `Loadable` mechanism, and GL enum-mapping helpers. `render_target.rs` holds `RenderTargetId` and the generation-checked slot table behind the offscreen cache surfaces.

Instanced sprite and mesh batches are backed by `ArrayBuffer<T>`, a growable array over one `ARRAY_BUFFER` whose mutations (`push`, `set`, swap-remove) are staged on the CPU in a `StagedVec<T>` and uploaded on `flush()` — one `bufferSubData` over the changed element range, or one `bufferData` reallocation (at least double the old capacity and 1.5× the new length) plus a full upload when the contents outgrow the buffer. `UnbindBatch` is the flush point, so a recording of N instance writes costs one GL upload instead of N bind/upload/unbind triples; draws use the flushed length (`gpu_len`), never unflushed elements. The buffer object never changes, so VAO bindings stay valid across growth. (The earlier immediate-upload design needed a GPU scratch buffer for swap-remove — see [pitfall/001](../pitfall/001_arraybuffer_swap_remove_buffer_binding_violation.md); staging on the CPU removed that path.) Per-instance GPU data is fixed-layout (`SpriteInstanceData` 72 bytes, `MeshInstanceData` 56 bytes, both including a per-instance tint and depth), and the byte offsets the VAO attribute setup reads from those structs are hardcoded rather than computed — see [pitfall/002](../pitfall/002_gpu_instance_struct_field_reorder_desync.md) for exactly what is and isn't guarded against a future field change. VAO attribute bindings are configured once at batch create/unbind time and simply bound (not reconfigured) at each draw call.

Depth ordering uses the WebGL2 depth buffer (`DEPTH_TEST` enabled, `LEQUAL` comparison so identical-depth draws fall back to submission order rather than rejecting the later one). `Transform::depth` is mapped into clip-space `z` by dividing by `RenderConfig::max_depth` (default `1.0`); the valid per-field range is `[-max_depth, max_depth]` and the GPU clips values outside it rather than the adapter validating them. For batch instances, the constraint applies to the **sum** `parent_depth + instance_depth`, not each independently. This ordering is reliable only for fully opaque draws — translucent content must still be submitted back-to-front by the caller, since alpha blending order is not resolved by the depth buffer. `DEPTH_TEST` also depends on the caller having created the WebGL2 context with a depth attachment; the standard `getContext` default (`depth: true`) provides one, but a caller-supplied context created with `depth: false` makes `DEPTH_TEST` a silent no-op — depth ordering then falls back to submission order with no error (confirmed via the depth-buffer doc comment in `src/adapters/webgl.rs`).

**Camera.** `WebGlBackend::camera_set` takes a column-major 3×3 world→screen view matrix, identity by default. Every world-space draw (`Mesh`, `Sprite`, both batch kinds, render-target and tiled-image quads) gets it multiplied in on the CPU as `mat3_mul( camera, transform )`, so the transform applies first; a batch's instances get it through the batch's parent matrix, which is computed at draw time, so a pan or zoom costs one uniform per batch and never re-uploads instance data. `ScreenSpaceSprite` skips it. It is an inherent method, not part of `Backend`, and no command carries a camera, so the other adapters draw world coordinates as viewport pixels (see the readme's capability table).

Three controls refine that default. `SetDepthWrite` toggles `depthMask`, so a caller can draw an opaque pass with depth writes on (nearer fragments early-Z reject farther ones) and then a transparent pass with writes off; the default is on, `Clear` forces it back on, and a caller that turns it off must restore it before the frame ends. A sprite batch with `SpriteBatchParams::occlude_overlap` draws under `LESS` instead of `LEQUAL` (restored after the draw), so once one instance writes the batch's depth at a pixel, overlapping instances at the same depth are rejected and each pixel blends once. The adapter does not clear depth for it: the caller keeps depth writes on and gives the batch a depth strictly nearer than whatever it should cover. A batch with `alpha_clip > 0` uses the cutout variant of `sprite_batch.frag`, which discards texels below the threshold (no colour, no depth write); batches with `alpha_clip == 0` use a discard-free variant so the GPU keeps early-Z. Single `Sprite` draws have no clip and no `LESS` mode.

Image upload has two paths with different synchronicity: `ImageSource::Bitmap` (already-decoded bytes) uploads synchronously via a direct `tex_image_2d` call; `ImageSource::Path` loads asynchronously through an `HtmlImageElement`, using `Closure::once_into_js` for one-shot `onload`/`onerror` handlers so the browser releases the captured Rust closure (and the `Rc<RefCell<GpuResources>>` it holds) once the event fires — this is what lets dropping a `WebGlBackend` actually release its GPU resources rather than leaking them via a retained closure cycle.

Blend modes `Normal`, `Add`, `Multiply`, and `Screen` map directly to a single `blend_func`/`blend_equation` call each and are hardware-accelerated; `BlendMode::Overlay` (the Photoshop-style conditional blend) cannot be expressed as one `blend_func` call and currently falls back to `Normal` — silently, from the caller's point of view, unless it inspects `capabilities().supported_blend_modes`, which deliberately omits `Overlay` from its four-entry list. `capabilities().blend_modes` is `false` specifically to signal that not every `BlendMode` variant renders correctly on this backend.

Premultiplied-alpha images (`ImageAsset::premultiplied`) are honoured here and nowhere else. An asynchronously loaded premultiplied image is uploaded with `UNPACK_COLORSPACE_CONVERSION_WEBGL` set to `NONE`, since the browser's default conversion of a colour-tagged file assumes straight colour. `GpuTexture` records the flag at upload; every draw path (single sprite, single mesh, sprite batch, mesh batch) reads it from its bound texture — an untextured mesh counts as straight — and passes it to `blend_apply`, which swaps the colour source factor of `Normal`, `Add` and the `Overlay` fallback from `SRC_ALPHA` to `ONE`. `Multiply` and `Screen` keep their factors, which read the source colour as stored: for a straight source below full alpha they approximate the reference formula (see `blend_apply`), for a premultiplied one they compute it exactly, so there a premultiplied image and its straight twin look different. The same flag is uploaded as the `u_premultiplied` uniform. Under it every fragment shader tints through the shared `shaders/tint.glsl`, which returns what the straight twin writes (colour and alpha each clamped to 0..1, as the RGBA8 target clamps them) with the colour multiplied by that alpha; without it the `ONE` factor would composite faded sprites, brightening tints and tint alphas above 1 too bright. `tests/webgl_premultiplied_test.rs` pins every draw path by pixel read-back: under `Normal`, `Add` and the `Overlay` fallback against its straight-alpha twin, under `Multiply` and `Screen` against the reference formula.

Command families the adapter does not yet implement — path (`BeginPath`..`EndPath`), text (`BeginText`..`EndText`), and group (`BeginGroup`..`EndGroup`) commands — are not fully silent: each family logs one `console::warn_1` diagnostic per `BeginPath`/`BeginText`/`BeginGroup` occurrence (referencing the relevant `capabilities()` flag), deliberately not per sub-command — a 1000-segment path still produces one warning, not 1000, since only the opener matches a warning arm and `MoveTo`/`LineTo`/`Char`/etc. are silent no-ops. There is no dedup across multiple openers in one stream or across `submit()` calls: five separate `BeginPath`...`EndPath` runs still produce five warnings. This is more precise than treating the whole family as unconditionally silent: there is a visible (browser-console) signal per path/text-run/group, just not a `RenderError` or a change to the rendered output.

Given the number of unimplemented command families and the `Overlay` blend-mode gap, this adapter's status is tracked as partial (⚠️), matching its characterization in `roadmap.md` ("WebGL2 adapter is partially implemented").

### Invariants

| File | Relationship |
|------|--------------|
| [invariant/001_y_up_coordinate_system.md](../invariant/001_y_up_coordinate_system.md) | Satisfied natively (OpenGL's own convention); no adapter-side conversion |
| [invariant/003_z_layer_draw_ordering.md](../invariant/003_z_layer_draw_ordering.md) | Additionally honors `Transform::depth` via `DEPTH_TEST`/`LEQUAL` (`LESS` for `occlude_overlap` batches), reliable only for opaque draws |
| [invariant/004_vector_representability_of_commands.md](../invariant/004_vector_representability_of_commands.md) | GPU backend accelerating the same declarative stream; adds nothing unrepresentable |

### Patterns

| File | Relationship |
|------|--------------|
| [pattern/001_ports_and_adapters_backend_architecture.md](../pattern/001_ports_and_adapters_backend_architecture.md) | This adapter is one `Backend` implementation within the crate's hexagonal architecture |

### Pitfalls

| File | Relationship |
|------|--------------|
| [pitfall/001_arraybuffer_swap_remove_buffer_binding_violation.md](../pitfall/001_arraybuffer_swap_remove_buffer_binding_violation.md) | Why a GPU-side swap-remove must not copy a buffer onto itself (historical — swap-remove is CPU-staged now) |
| [pitfall/002_gpu_instance_struct_field_reorder_desync.md](../pitfall/002_gpu_instance_struct_field_reorder_desync.md) | Hardcoded VAO offsets vs. size/align-only compile-time assertions |

### Sources

| File | Relationship |
|------|--------------|
| `src/adapters/webgl.rs` | `WebGlBackend`: command dispatch, asset loading, context-loss handling |
| `src/adapters/webgl/webgl_renderers.rs` | `SpriteRenderer` / `MeshRenderer`: shader programs, the `tint.glsl` splice, draw calls |
| `src/adapters/webgl/webgl_textures.rs` | `bitmap_texture_upload`, `image_upload_from_path` (async image loader) |
| `src/adapters/webgl/webgl_helpers.rs` | `ArrayBuffer<T>`, instance-data layouts, GPU resource handles, VAO setup, GL mapping helpers |
| `src/adapters/webgl/render_target.rs` | `RenderTargetId` and the generation-checked slot table behind the offscreen cache surfaces |
| `src/adapters/webgl/webgl_surfaces.rs` | `WebGlBackend`'s render-target lifecycle, `scissor_set`, `fill`, `render_target_draw`, `image_tiled_draw` |
| `src/adapters/shaders/sprite.vert`, `sprite.frag` | Single-sprite shader pair |
| `src/adapters/shaders/sprite_batch.vert`, `sprite_batch.frag` | Instanced sprite-batch shader pair |
| `src/adapters/shaders/mesh.vert`, `mesh.frag` | Single-mesh shader pair |
| `src/adapters/shaders/mesh_batch.vert`, `mesh_batch.frag` | Instanced mesh-batch shader pair |
| `src/adapters/shaders/tint.glsl` | The tint all four fragment shaders apply, straight or premultiplied, spliced in at their `#include "tint.glsl"` line |

### Tests

| File | Relationship |
|------|--------------|
| `tests/webgl_helpers_test.rs` | Host tests of the GL-free `StagedVec` bookkeeping behind `ArrayBuffer` (one range per batch of writes, each arm of the realloc growth policy, nothing recorded until an upload succeeds, the `i32` size limit, merged set/swap-remove ranges, no upload for tail removes) and of `mat3_mul`, which folds the camera into each draw (`test_internals`) |
| `tests/webgl_render_target_test.rs` | Host tests of the GL-free slot table behind `RenderTargetId`: a handle kept across `render_targets_reset` is rejected even after its slot is reused (`test_internals`) |
| `tests/webgl_array_buffer_test.rs` | Live WebGL2 context: `ArrayBuffer::flush` uploads what `StagedVec` plans, and `gpu_len` counts only flushed elements (GPU contents read back) |
| `tests/webgl_camera_test.rs` | Live WebGL2 context, pixel read-back: `camera_set` moves a `Mesh`, a `Sprite` and both batch kinds like the camera folded in by hand, applies after a batch's parent, moves a retained batch with no re-upload, and leaves a `ScreenSpaceSprite` alone |
| `tests/webgl_depth_coverage_test.rs` | Live WebGL2 context, pixel read-back: `SetDepthWrite( false )` leaves no depth, `occlude_overlap` paints an overlap once, `alpha_clip` discards colour and depth below the threshold |
| `tests/webgl_backend_test.rs` | Compile-and-construct-level coverage: `declared_capabilities`'s honest-subset pin (`meshes`/`sprites`/`batches`/`premultiplied_images`/`depth` true, everything else false, exact 4-entry `supported_blend_modes`) and an anti-hardcoding pin (two different `max_texture_size` inputs produce two different, input-matching outputs). No `WebGl2RenderingContext`/`web_sys` call; the live-context suites below cover what needs one |
| `tests/webgl_premultiplied_test.rs` | Live WebGL2 context, pixel read-back: premultiplied compositing on every draw path against the straight-alpha twin (`Normal`, `Add`, `Overlay` fallback, tint alpha, out-of-range tints) or the reference formula (`Multiply`, `Screen`), and an encoded premultiplied image uploaded without colour conversion |
| `tests/webgl_pending_image_test.rs` | Live WebGL2 context, pixel read-back: a textured mesh / mesh batch on an image still decoding is skipped, then draws once it lands |
| `tests/webgl_context_loss_test.rs` | Live WebGL2 context: `context_lost` blocks `submit` / `output` until `assets_load` clears it, and a real `WEBGL_lose_context` loss and restore leaves the backend drawing again (`test_internals`) |
| `tests/manual/readme.md` | Real-browser (`browsee`) pixel verification beyond compile-and-construct-level — confirms `WebGlBackend::new` + `assets_load` + `submit` + `output` paints a correctly-centered `rgb 255 0 0` sprite on a `rgb 0 0 255` clear through a live WebGL2 context (Firefox) |

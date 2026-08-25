# Feature: Command Recording & Submission

### Scope

- **Purpose**: Record one render pass's draw calls and submit the encoded work to the GPU queue; write data directly into an existing buffer or texture outside a pass.
- **Responsibility**: Document the command-encoding, draw-recording, and queue-submission API's design.
- **In Scope**: `command_encoder_create`, `render_pass_begin`, `ColorAttachmentDesc`/`DepthAttachmentDesc`, `RenderPass` methods (`pipeline_set`/`bind_group_set`/`vertex_buffer_set`/`index_buffer_set`/`draw`/`draw_indexed`/`end`), `Queue::submit`, `Queue::buffer_write`/`texture_write`.
- **Out of Scope**: Pipeline and bind-group *construction* consumed during a pass (see `feature/003`, `feature/004`); reading rendered pixels back on the native and Vulkan backends (see `feature/006`).

### Design

`Device::command_encoder_create()` returns a `CommandEncoder` for one frame's passes with a `Result`-free signature on every backend; on Vulkan specifically, that same call can still panic on a genuine, caller-independent driver failure rather than return an error — a deliberate signature tradeoff, not an oversight (see `invariant/001`'s Category B). `CommandEncoder::render_pass_begin(&ColorAttachmentDesc, Option<&DepthAttachmentDesc>)` begins one render pass — one color attachment plus an optional depth attachment, always-clear load ops, both attachments required to be texture views of matching size — and returns a `RenderPass`. It fails with `Error::WebGpu` on a WebGPU pass-creation failure; on WebGL it returns `Error::Unsupported` if a depth attachment is paired with the canvas backbuffer (either as the color target or as the depth view itself — the backbuffer accepts no depth attachment) or `Error::WebGl` if the backing framebuffer fails to allocate; native never fails this call; on Vulkan it returns `Error::Vulkan` if the underlying render pass or framebuffer creation fails.

Every `RenderPass` recording method (`pipeline_set`, `bind_group_set`, `vertex_buffer_set`, `index_buffer_set`, `draw`, `draw_indexed`) takes `&mut self`, because the native backend records into its raw `wgpu` pass mutably and the browser backends share the same signature; `end(self)` consumes the pass. The WebGL backend applies state eagerly against the active pipeline's introspected binding maps, which is what makes `pipeline_set`-before-`bind_group_set`/`vertex_buffer_set` a real ordering requirement rather than a style preference (see `invariant/002`).

`Queue::submit(encoder: CommandEncoder)` finishes the encoder and submits its command buffer in one call — it consumes the encoder by value, since both WebGPU's and `wgpu`'s own `finish()` take ownership and a submitted encoder must not be reusable afterward. Like `command_encoder_create`, `submit` keeps a `Result`-free signature on every backend, but on Vulkan can panic on a genuine driver failure from any of its three underlying calls (`vkEndCommandBuffer`/`vkQueueSubmit`/`vkQueueWaitIdle` — see `invariant/001`'s Category B). Fix(BUG-470): on Vulkan, once `vkQueueWaitIdle` confirms the GPU has finished executing the encoder, `submit` also destroys the command pool `command_encoder_create` allocated and every render pass/framebuffer pair `render_pass_begin` created on it — the only safe point to do so, since destroying them any earlier would free objects a not-yet-completed command buffer still references. Unlike the opt-in `Device::buffer_destroy`/`texture_destroy`/etc. methods (`src/device.rs`), this destruction is unconditional and transparent to every caller — there is no `command_encoder_destroy` to opt into, since no API ever hands the pool back as a separately owned resource. Outside a pass, `Queue::buffer_write(&buffer, data)` writes bytes at offset zero — every backend can fail it: `Error::WebGpu` if the underlying WebGPU write call fails; `Error::InvalidInput` on WebGL or native if `data` overruns the buffer's allocated size (native additionally rejects a length that isn't a multiple of wgpu's `COPY_BUFFER_ALIGNMENT`); or on Vulkan either that same `Error::InvalidInput` oversized-write case or `Error::Vulkan` if the underlying `vkMapMemory` call fails. `Queue::texture_write(&texture, data)` uploads a tightly-packed CPU-side buffer, failing with `Error::Unsupported` if the texture's format has no portable CPU-side texel layout (e.g. `Depth24Plus` — see `feature/002`'s `bytes_per_texel`) or a backend-specific error if the underlying write call fails; the WebGPU and native arms derive their own row alignment internally, WebGL and Vulkan never need one.

### Invariants

| File | Relationship |
|------|--------------|
| [invariant/001_result_based_error_handling_scoped_panics.md](../invariant/001_result_based_error_handling_scoped_panics.md) | Every fallible call here returns `Result<_, Error>`, except Vulkan's two intentionally-infallible-signature panics on `command_encoder_create`/`submit` (Category B) |
| [invariant/002_webgl_render_pass_recording_order.md](../invariant/002_webgl_render_pass_recording_order.md) | This feature's `RenderPass` methods are exactly what that ordering invariant constrains |

### Patterns

| File | Relationship |
|------|--------------|
| [pattern/001_enum_per_backend_dispatch_one_step_drilldown.md](../pattern/001_enum_per_backend_dispatch_one_step_drilldown.md) | `CommandEncoder`/`RenderPass` are backend-tagged enums like every other handle |

### Sources

| File | Relationship |
|------|--------------|
| `src/pass.rs` | `CommandEncoder`, `RenderPass`, `ColorAttachmentDesc`/`DepthAttachmentDesc` |
| `src/device.rs` | `Queue::buffer_write`/`texture_write`/`submit`, `command_encoder_create` |
| `src/vulkan.rs` | Vulkan arms: `render_pass_begin`, `buffer_write`, `texture_write`, and `submit`'s three panicking driver calls |

### Tests

| File | Relationship |
|------|--------------|
| `tests/native_backend_test.rs` | All three tests record `pipeline_set → bind_group_set → vertex_buffer_set → index_buffer_set → draw_indexed → end` then `queue.submit(encoder)` in that order |
| `tests/vulkan_backend_test.rs` | `triangle_render_readback` exercises the same recording order plus `queue.buffer_write` on an empty `buffer_create`d uniform buffer, proving the write itself lands rather than just `buffer_init_create` |

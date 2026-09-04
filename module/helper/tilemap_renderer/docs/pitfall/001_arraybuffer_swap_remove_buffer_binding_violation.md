# Pitfall: ArrayBuffer Swap-Remove Buffer-Binding Violation

### Scope

- **Purpose**: Record why `ArrayBuffer<T>::swap_remove` cannot copy directly from a GPU buffer to itself.
- **Responsibility**: Document the WebGL2 spec constraint, the observed failure it would cause, and the mitigation already in place.
- **In Scope**: `ArrayBuffer<T>::swap_remove`'s GPU-to-GPU copy step in the WebGL2 adapter.
- **Out of Scope**: `ArrayBuffer<T>`'s grow-on-full behavior (`copy_buffer_sub_data` into a freshly allocated buffer), which copies between two distinct buffers and is not subject to this constraint.

### Trap

Assuming a same-buffer, self-to-self `copy_buffer_sub_data` call — binding one `WebGlBuffer` to both `COPY_READ_BUFFER` and `COPY_WRITE_BUFFER` at once — is a valid way to move the last element into a removed slot during an instanced-batch swap-remove.

### Failure

The WebGL2 spec disallows binding the same buffer object to `COPY_READ_BUFFER` and `COPY_WRITE_BUFFER` simultaneously; doing so raises `INVALID_OPERATION`. A naive swap-remove implementation — copy the last element's bytes directly over the removed element's bytes within the same `ArrayBuffer` — would trigger this on every removal from a non-tail index, since both the source and destination ranges live in the same underlying `WebGlBuffer`.

### Mitigation

`ArrayBuffer<T>` (`src/adapters/webgl/webgl_helpers.rs`) allocates a **persistent one-element scratch buffer** (`scratch : web_sys::WebGlBuffer`, created alongside the main buffer and freed in `Drop`) and routes `swap_remove` through it in two GPU-to-GPU copies instead of one: last element → scratch (`COPY_READ_BUFFER` = main buffer, `COPY_WRITE_BUFFER` = scratch), then scratch → removed slot (`COPY_READ_BUFFER` = scratch, `COPY_WRITE_BUFFER` = main buffer). Both copies bind two distinct buffer objects, so the spec constraint is satisfied and the whole operation stays GPU-side with no CPU readback. This is fully mitigated — the scratch-buffer indirection is unconditional in `swap_remove`, not a fallback path.

### Features

| File | Relationship |
|------|--------------|
| [feature/002_webgl2_backend_adapter.md](../feature/002_webgl2_backend_adapter.md) | `ArrayBuffer<T>` backs the GPU-side instance storage for sprite and mesh batches |

### Sources

| File | Relationship |
|------|--------------|
| `src/adapters/webgl/webgl_helpers.rs` | `ArrayBuffer::swap_remove` and the `scratch` buffer it uses as an intermediary |

### Tests

| File | Relationship |
|------|--------------|
| — | No automated test currently exercises the WebGL2 adapter (it requires a `wasm32` target and a live WebGL2 context); see [feature/002_webgl2_backend_adapter.md](../feature/002_webgl2_backend_adapter.md) for the crate's WebGL2 test-coverage status |

# Feature: Buffer Management

### Scope

- **Purpose**: Create and initialize GPU buffers while transparently satisfying WebGPU's buffer-alignment requirements.
- **Responsibility**: Document the buffer construction API's design.
- **In Scope**: Buffer creation and data-initializing construction.
- **Out of Scope**: Binding buffers into bind groups (see `feature/004`).

### Design

Two entry points cover buffer creation: `buffer::create` wraps `GpuDevice::create_buffer` for an empty/uninitialized buffer from a caller-supplied `GpuBufferDescriptor`. `buffer::init` additionally uploads initial data — it rounds the requested size up to `COPY_BUFFER_ALIGNMENT`, creates the buffer with `mapped_at_creation` set, copies the caller's bytes into the mapped range through a `Uint8Array` view, and unmaps it. Both paths convert `web-sys` JS errors into `WebGPUError::BufferError`/`DeviceError` variants (see `invariant/001`). Callers needing typed data must implement `AsBytes` (from the `asbytes` crate) for `init`'s generic payload.

### Patterns

| File | Relationship |
|------|--------------|
| [pattern/001_facade_over_descriptor_builders.md](../pattern/001_facade_over_descriptor_builders.md) | Buffer creation follows the crate's descriptor-plus-explicit-device shape |

### Invariants

| File | Relationship |
|------|--------------|
| [invariant/001_result_based_error_handling.md](../invariant/001_result_based_error_handling.md) | All fallible functions here return `Result<_, WebGPUError>` |

### Sources

| File | Relationship |
|------|--------------|
| `src/buffer.rs` | `create`/`init` buffer construction |
| `src/descriptor/buffer.rs` | `GpuBufferDescriptor` builder |
| `src/descriptor/buffer_init.rs` | `BufferInitDescriptor` builder |
| `src/mem.rs` | Low-level data manipulation used during initialization |

### Tests

No automated tests exist for this crate at the time of this migration.

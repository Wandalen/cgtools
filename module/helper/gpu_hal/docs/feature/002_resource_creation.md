# Feature: Resource Creation

### Scope

- **Purpose**: Create the GPU-memory-owning handle types — buffers, textures, samplers — a `Device` can hand out.
- **Responsibility**: Document the resource-creation API's design.
- **In Scope**: `buffer_create`, `buffer_init_create`, `texture_create`, `sampler_create`; the `Buffer`/`Texture`/`TextureView`/`Sampler` handle types; `BufferUsage`/`TextureUsage`/`TextureDesc`/`SamplerDesc`/`FilterMode`/`AddressMode`/`TextureFormat`.
- **Out of Scope**: Shader modules and pipelines (see `feature/003`); bind group construction that consumes these handles (see `feature/004`); writing data into an existing buffer/texture, which is a `Queue` operation (see `feature/005`).

### Design

Two buffer constructors mirror the empty-vs-initialized split seen in sibling HAL crates: `buffer_create(size, usage)` allocates an uninitialized buffer of `size` bytes; `buffer_init_create(data, usage)` allocates and uploads `data` in the same call. Both take a `BufferUsage` bit-flag value (`COPY_DST`/`INDEX`/`VERTEX`/`UNIFORM`) and fail on the WebGPU and WebGL backends (`Error::WebGpu`/`Error::WebGl` on an underlying allocation failure) or on Vulkan (`Error::Vulkan` if the underlying memory allocation or buffer creation fails) — native never fails either call. `texture_create(&TextureDesc)` allocates a 2D texture (`size: [u32; 3]` width/height/depth-or-layers, one mip, one sample — the v0 surface has no mip or MSAA support) from a `TextureFormat` and `TextureUsage` (`COPY_DST`/`TEXTURE_BINDING`/`RENDER_ATTACHMENT`); it additionally fails on WebGL if `desc.format` has no WebGL internal-format mapping, or on Vulkan if format resolution or the underlying image allocation fails. `sampler_create(SamplerDesc)` takes a plain-data `{ filter: FilterMode, address: AddressMode }` pair whose `Default` mirrors WebGPU's own defaults (`Nearest` filtering, `ClampToEdge` addressing) and fails on WebGL allocation failure or on Vulkan if the underlying `vkCreateSampler` call fails.

`TextureFormat` covers the v0 surface's five formats (`Rgba8Unorm`, `Rgba8UnormSrgb`, `Bgra8Unorm`, `Rgba16Float`, `Depth24Plus`); `bytes_per_texel()` returns the tightly-packed CPU-side byte width for the first four, and `Error::Unsupported` for `Depth24Plus`, whose CPU-side layout is platform-defined and not a portable upload target — the same distinction `feature/005`'s `texture_write` relies on.

### Invariants

| File | Relationship |
|------|--------------|
| [invariant/001_result_based_error_handling_scoped_panics.md](../invariant/001_result_based_error_handling_scoped_panics.md) | All four constructors return `Result<_, Error>` |

### Patterns

| File | Relationship |
|------|--------------|
| [pattern/001_enum_per_backend_dispatch_one_step_drilldown.md](../pattern/001_enum_per_backend_dispatch_one_step_drilldown.md) | Every handle type returned here is a backend-tagged enum with `as_webgpu`/`as_webgl`/`as_native`/`as_vulkan` drill-downs |

### Sources

| File | Relationship |
|------|--------------|
| `src/device.rs` | `buffer_create`, `buffer_init_create`, `texture_create`, `sampler_create` |
| `src/resource.rs` | `Buffer`/`Texture`/`TextureView`/`Sampler` enums and their `as_webgpu`/`as_webgl`/`as_native`/`as_vulkan` accessors |
| `src/types.rs` | `BufferUsage`, `TextureUsage`, `TextureDesc`, `SamplerDesc`, `FilterMode`, `AddressMode`, `TextureFormat` |
| `src/vulkan.rs` | Vulkan arms: `buffer_create`, `buffer_init_create`, `texture_create`, `sampler_create` |

### Tests

| File | Relationship |
|------|--------------|
| `tests/native_backend_test.rs` | `buffer_init_create` (vertex/index buffers), `buffer_create` (uniform buffer), `texture_create`/`sampler_create` (`textured_bind_group_create`) exercised throughout `triangle_render_readback` and `texture_write_readback` |
| `tests/vulkan_backend_test.rs` | `triangle_render_readback` exercises `buffer_init_create` (vertex/index buffers) and `buffer_create` (uniform buffer); `vulkan_texture_write_readback` exercises `texture_create`/`sampler_create` (`textured_bind_group_create`) |

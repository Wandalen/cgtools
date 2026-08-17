# Feature: Bind Groups & Layouts

### Scope

- **Purpose**: Describe binding-slot types and shader visibility (`BindGroupLayout`), then bind concrete resources to those slots (`BindGroup`) so shaders can access buffers, textures, and samplers.
- **Responsibility**: Document the bind-group construction API's design, including the WebGL-specific entry-order constraint.
- **In Scope**: `bind_group_layout_create`, `bind_group_create`, `BindGroupLayoutEntry`, `BindingType`, `BindingResource`, `ShaderStages`.
- **Out of Scope**: Pipeline construction that consumes the resulting `BindGroupLayout` (see `feature/003`); binding the created `BindGroup` during a pass (see `feature/005`).

### Design

`bind_group_layout_create(&[BindGroupLayoutEntry])` takes a slice of `{ visibility: ShaderStages, ty: BindingType }` entries — binding indices follow entry order, there is no explicit index field. `BindingType` covers `UniformBuffer`/`Texture`/`Sampler`; `ShaderStages` covers `VERTEX`/`FRAGMENT`. It fails with `Error::WebGpu` if the underlying WebGPU entry- or layout-creation call fails (see `invariant/001`'s worked `Fix(BUG-051)` example), or `Error::Vulkan` if the underlying `vkCreateDescriptorSetLayout` call fails — WebGL and native never fail this call. `bind_group_create(&layout, &[BindingResource])` pairs the layout with concrete `BindingResource::Buffer`/`TextureView`/`Sampler` references, in the same entry order as the layout; it returns `Error::Unsupported` on WebGL specifically if a sampled texture view is the canvas backbuffer itself (the backbuffer cannot be sampled), or `Error::Vulkan` if the underlying descriptor pool creation or descriptor set allocation fails — WebGPU and native never fail this call.

The WebGL backend resolves bindings by GLSL name convention rather than an explicit numeric slot: a uniform block is named `ub_{group}_{binding}`, a sampler uniform `tex_{group}_{binding}` (see `invariant/003`). Within one group, a `Sampler` entry pairs with the **nearest preceding** `Texture` entry — entry order in the slice passed to both constructors is therefore load-bearing on WebGL specifically, not just a bookkeeping convenience (see `invariant/003`).

### Invariants

| File | Relationship |
|------|--------------|
| [invariant/001_result_based_error_handling_scoped_panics.md](../invariant/001_result_based_error_handling_scoped_panics.md) | Both constructors return `Result<_, Error>` |
| [invariant/003_webgl_bind_group_entry_order.md](../invariant/003_webgl_bind_group_entry_order.md) | This feature's entry slice is exactly what that ordering invariant constrains |

### Patterns

| File | Relationship |
|------|--------------|
| [pattern/001_enum_per_backend_dispatch_one_step_drilldown.md](../pattern/001_enum_per_backend_dispatch_one_step_drilldown.md) | `BindGroupLayout`/`BindGroup` are backend-tagged enums like every other handle |

### Sources

| File | Relationship |
|------|--------------|
| `src/device.rs` | `bind_group_layout_create`, `bind_group_create` |
| `src/resource.rs` | `BindGroupLayout`/`BindGroup` enums, `BindingResource` |
| `src/types.rs` | `BindGroupLayoutEntry`, `BindingType`, `ShaderStages` |
| `src/webgl.rs` | GLSL binding-name convention and nearest-preceding-texture pairing |
| `src/vulkan.rs` | Vulkan arms: `bind_group_layout_create`, `bind_group_create` |

### Tests

| File | Relationship |
|------|--------------|
| `tests/native_backend_test.rs` | `textured_bind_group_create`'s inline comment states the texture-before-sampler order is "load-bearing" even though the test itself runs under the native backend, which binds explicitly and is unaffected by order |
| `tests/vulkan_backend_test.rs` | `triangle_render_readback` exercises `bind_group_layout_create`/`bind_group_create` with a `UniformBuffer` entry; `vulkan_texture_write_readback` exercises the same pair with `Texture`/`Sampler` entries in the same texture-before-sampler order |

# Feature: Buffer and Texture Builders

`minwgpu` provides fluent builders for `wgpu::Buffer` and vertex buffers (`buffer()` / `vertex_buffer()`), plus a plain `Texture` struct that bundles a `wgpu::Texture` with its view, sampler, and extent.

### Scope

- **Purpose**: Reduce the boilerplate of `wgpu` buffer/vertex-buffer descriptor construction and keep a texture's associated view/sampler/extent together.
- **Responsibility**: Cross-reference the source and tests that make up buffer and texture construction.
- **In Scope**: `BufferBuilder`, `VertexBufferBuilder`, `VertexBuffer`, `Texture`, and the `attr` vertex-attribute helper.
- **Out of Scope**: `wgpu` context/device/queue setup (see [feature/001_context_builder.md](001_context_builder.md)).

### Design

`buffer( usage )` returns a `BufferBuilder` for a generic `wgpu::Buffer`; `vertex_buffer()` returns a `VertexBufferBuilder` pre-configured with `BufferUsages::VERTEX` and `VertexStepMode::Vertex`, additionally exposing `array_stride`, `step_mode`, and `attributes` to describe a `wgpu::VertexBufferLayout`. Both builders share the same configuration methods (`data`, `label`, `size`, `size_from_var`, `size_from_value`, `mapped_at_creation`, `usage`, `vertex_usage`) via one macro (`impl_buffer_builder_methods!`) applied to each builder's inner state.

`.build( device )` picks one of two `wgpu` construction paths depending on whether `.data(..)` was called: if data is present, the buffer is created via `wgpu::util::DeviceExt::create_buffer_init` from that data, and the separately-configured `.size(..)` / `.mapped_at_creation(..)` are not used; otherwise it is created via `wgpu::Device::create_buffer` using the configured `.size(..)` / `.mapped_at_creation(..)`. `VertexBufferBuilder::build( device )` additionally pairs the built `wgpu::Buffer` with a `wgpu::VertexBufferLayout` assembled from `.array_stride(..)`, `.step_mode(..)`, and `.attributes(..)`, returning a `VertexBuffer`.

`attr( format, offset, shader_location )` (`src/helper.rs`) is a `const fn` shortcut for constructing one `wgpu::VertexAttribute`, used to build the slice passed to `.attributes(..)`.

`Texture` (`src/texture.rs`) is not a builder — it is a `#[non_exhaustive]` struct that bundles an already-constructed `wgpu::Texture`, its `wgpu::Extent3d`, `wgpu::TextureView`, and `wgpu::Sampler` behind a single `Texture::new(..)` constructor, so the four related values can be passed and stored as one unit.

### Sources

| File | Relationship |
|------|--------------|
| `src/buffer.rs` | `BufferBuilder` / `VertexBufferBuilder` / `VertexBuffer` implementation |
| `src/texture.rs` | `Texture` bundling struct |
| `src/helper.rs` | `attr` vertex-attribute helper |

### Tests

| File | Relationship |
|------|--------------|
| `src/buffer.rs` | Inline `#[cfg(test)]` coverage of `BufferBuilder` / `VertexBufferBuilder` configuration methods and defaults |

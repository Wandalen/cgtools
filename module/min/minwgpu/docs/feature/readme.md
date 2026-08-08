# Feature Doc Definition

### Scope

- **Purpose**: `minwgpu`'s builders exist to remove `wgpu` setup and resource-construction boilerplate without hiding the underlying `wgpu` types.
- **Responsibility**: Document each builder-based capability as a navigational hub over its source and tests.
- **In Scope**: Context/adapter/device construction and buffer/vertex-buffer/texture construction exposed by `minwgpu`'s public API.
- **Out of Scope**: Implementation-level `wgpu` descriptor detail (see the Sources references inside each instance).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [Context Builder](001_context_builder.md) | Type-state builder for `wgpu` instance/adapter/device/queue setup | ✅ |
| 002 | [Buffer and Texture Builders](002_buffer_and_texture_builders.md) | Fluent builders for `wgpu` buffers/vertex buffers, plus a texture bundle | ✅ |

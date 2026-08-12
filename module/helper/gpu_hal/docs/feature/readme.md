# Feature Doc Definition

A **feature** instance documents one cohesive slice of the crate's public API. In `gpu_hal`, features are grouped by HAL concern — backend construction, resource creation, shader/pipeline setup, bind groups, command recording, and native pixel readback — with each entry linking out to its source, pattern, and invariants. This collection holds one instance per feature; the table below is the index into them.

### Scope

- **Purpose**: Navigational hub for `gpu_hal`'s public API surface, grouped by HAL concern.
- **Responsibility**: Document each API area's design and link to its source, pattern, and invariants.
- **In Scope**: Backend construction, resource creation, shader/pipeline setup, bind groups, command recording, native pixel readback.
- **Out of Scope**: Internal architecture rationale (see `pattern/`); the workspace-level layer contract (see `docs/layer/002_l1_gpu_hal.md`).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [Backend Construction & Device Acquisition](001_backend_construction_and_device_acquisition.md) | `Device`/`Queue`/`Surface` acquisition per backend, clip-space depth contract | ✅ |
| 002 | [Resource Creation](002_resource_creation.md) | Buffer, texture, and sampler construction | ✅ |
| 003 | [Shader Modules & Render Pipelines](003_shader_modules_and_render_pipelines.md) | WGSL/GLSL shader compilation and render pipeline assembly | ✅ |
| 004 | [Bind Groups & Layouts](004_bind_groups_and_layouts.md) | Binding-slot layout description and resource binding | ✅ |
| 005 | [Command Recording & Submission](005_command_recording_and_submission.md) | Render pass recording and queue submission | ✅ |
| 006 | [Native Pixel Readback](006_native_pixel_readback.md) | GPU→CPU pixel readback on the native backend only | ✅ |

# Feature Doc Definition

A **feature** instance documents one cohesive slice of the crate's public API. In `minwebgpu`, features are grouped by WebGPU concern — context/device/shader setup, buffer management, pipeline management, resource binding, and command recording — with each entry linking out to its source, pattern, and invariants. This collection holds one instance per feature; the table below is the index into them.

### Scope

- **Purpose**: Navigational hub for `minwebgpu`'s public API surface, grouped by WebGPU concern.
- **Responsibility**: Document each API area's design and link to its source, pattern, and invariants.
- **In Scope**: Context/device/shader setup, buffer management, pipeline management, resource binding, command recording.
- **Out of Scope**: Internal architecture rationale (see `pattern/`).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [Context, Device & Shader Setup](001_context_device_and_shader_setup.md) | Canvas/adapter/device/queue acquisition and WGSL shader compilation | ✅ |
| 002 | [Buffer Management](002_buffer_management.md) | GPU buffer creation with alignment-safe initialization | ✅ |
| 003 | [Pipeline Management](003_pipeline_management.md) | Render/compute pipeline descriptor builders | ✅ |
| 004 | [Resource Binding](004_resource_binding.md) | Bind group layouts and bind groups | ✅ |
| 005 | [Command Recording & Execution](005_command_recording_and_execution.md) | Render pass recording and queue submission (compute pass recording not implemented) | ⚠️ |

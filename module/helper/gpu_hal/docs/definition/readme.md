# Doc Definitions

## Master Doc Definitions Table

| Type | Purpose | Master File | Instances |
|------|---------|-------------|----------:|
| `feature/` | Public API surface grouped by HAL concern | [feature/readme.md](../feature/readme.md) | 6 |
| `invariant/` | Crate-wide error contract and WebGL ordering guarantees | [invariant/readme.md](../invariant/readme.md) | 3 |
| `pattern/` | Core architectural approach | [pattern/readme.md](../pattern/readme.md) | 1 |
| `pitfall/` | Confirmed cross-backend traps | [pitfall/readme.md](../pitfall/readme.md) | 2 |

## Master Doc Instances Table

| Definition | ID  | Name | File |
|---------|-----|------|------|
| feature | 001 | Backend Construction & Device Acquisition | [feature/001_backend_construction_and_device_acquisition.md](../feature/001_backend_construction_and_device_acquisition.md) |
| feature | 002 | Resource Creation | [feature/002_resource_creation.md](../feature/002_resource_creation.md) |
| feature | 003 | Shader Modules & Render Pipelines | [feature/003_shader_modules_and_render_pipelines.md](../feature/003_shader_modules_and_render_pipelines.md) |
| feature | 004 | Bind Groups & Layouts | [feature/004_bind_groups_and_layouts.md](../feature/004_bind_groups_and_layouts.md) |
| feature | 005 | Command Recording & Submission | [feature/005_command_recording_and_submission.md](../feature/005_command_recording_and_submission.md) |
| feature | 006 | Off-Browser Pixel Readback | [feature/006_native_pixel_readback.md](../feature/006_native_pixel_readback.md) |
| invariant | 001 | Result-Based Error Handling with a Scoped Panic Policy | [invariant/001_result_based_error_handling_scoped_panics.md](../invariant/001_result_based_error_handling_scoped_panics.md) |
| invariant | 002 | WebGL Render-Pass Recording Order | [invariant/002_webgl_render_pass_recording_order.md](../invariant/002_webgl_render_pass_recording_order.md) |
| invariant | 003 | WebGL Bind-Group Entry Order | [invariant/003_webgl_bind_group_entry_order.md](../invariant/003_webgl_bind_group_entry_order.md) |
| pattern | 001 | Enum-Per-Backend Dispatch with One-Step Drill-Down | [pattern/001_enum_per_backend_dispatch_one_step_drilldown.md](../pattern/001_enum_per_backend_dispatch_one_step_drilldown.md) |
| pitfall | 001 | Backend Availability Is Compile-Time, Not Runtime | [pitfall/001_backend_availability_compile_time_not_runtime.md](../pitfall/001_backend_availability_compile_time_not_runtime.md) |
| pitfall | 002 | Pixel Readback Is Unsupported on Browser Backends | [pitfall/002_pixel_readback_native_only.md](../pitfall/002_pixel_readback_native_only.md) |

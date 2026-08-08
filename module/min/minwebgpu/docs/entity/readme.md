# Doc Entities

## Master Doc Entities Table

| Type | Purpose | Master File | Instances |
|------|---------|-------------|----------:|
| `feature/` | Public API surface grouped by WebGPU concern | [feature/readme.md](feature/readme.md) | 5 |
| `invariant/` | Crate-wide safety/error-handling guarantees | [invariant/readme.md](invariant/readme.md) | 1 |
| `non_functional_requirement/` | Measurable quality-attribute targets | [non_functional_requirement/readme.md](non_functional_requirement/readme.md) | 1 |
| `pattern/` | Core architectural approach | [pattern/readme.md](pattern/readme.md) | 1 |
| `pitfall/` | Confirmed build/runtime traps | [pitfall/readme.md](pitfall/readme.md) | 1 |

## Master Doc Instances Table

| Entity  | ID  | Name | File |
|---------|-----|------|------|
| feature | 001 | Context, Device & Shader Setup | [feature/001_context_device_and_shader_setup.md](feature/001_context_device_and_shader_setup.md) |
| feature | 002 | Buffer Management | [feature/002_buffer_management.md](feature/002_buffer_management.md) |
| feature | 003 | Pipeline Management | [feature/003_pipeline_management.md](feature/003_pipeline_management.md) |
| feature | 004 | Resource Binding | [feature/004_resource_binding.md](feature/004_resource_binding.md) |
| feature | 005 | Command Recording & Execution | [feature/005_command_recording_and_execution.md](feature/005_command_recording_and_execution.md) |
| invariant | 001 | Result-Based Error Handling, No Unsafe Code | [invariant/001_result_based_error_handling.md](invariant/001_result_based_error_handling.md) |
| non_functional_requirement | 001 | Minimal Abstraction Overhead | [non_functional_requirement/001_minimal_abstraction_overhead.md](non_functional_requirement/001_minimal_abstraction_overhead.md) |
| pattern | 001 | Facade Over Descriptor Builders | [pattern/001_facade_over_descriptor_builders.md](pattern/001_facade_over_descriptor_builders.md) |
| pitfall | 001 | Native-Target Builds Compile to a Non-Functional Stub | [pitfall/001_native_target_compiles_to_nonfunctional_stub.md](pitfall/001_native_target_compiles_to_nonfunctional_stub.md) |

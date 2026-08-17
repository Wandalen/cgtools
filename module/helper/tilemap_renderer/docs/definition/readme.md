# Doc Definitions

## Master Doc Definitions Table

| Type | Purpose | Master File | Instances |
|------|---------|-------------|----------:|
| `feature/` | Backend adapters (SVG, WebGL2, Terminal, None, WebGPU, Native) as navigational hubs over source, invariants, patterns, and pitfalls | [feature/readme.md](../feature/readme.md) | 6 |
| `invariant/` | Cross-backend correctness guarantees (Y-up coordinates, SVG injection safety, draw ordering, vector representability) | [invariant/readme.md](../invariant/readme.md) | 4 |
| `pattern/` | The core/adapter (Ports and Adapters) architecture shared by all backends | [pattern/readme.md](../pattern/readme.md) | 1 |
| `pitfall/` | Confirmed GPU-buffer traps, their failure modes, and mitigations | [pitfall/readme.md](../pitfall/readme.md) | 2 |

## Master Doc Instances Table

| Definition | ID  | Name                                     | File                                                                                                             |
|-----------|-----|-------------------------------------------|--------------------------------------------------------------------------------------------------------------------|
| feature   | 001 | SVG Backend Adapter                       | [feature/001_svg_backend_adapter.md](../feature/001_svg_backend_adapter.md)                                          |
| feature   | 002 | WebGL2 Backend Adapter                    | [feature/002_webgl2_backend_adapter.md](../feature/002_webgl2_backend_adapter.md)                                    |
| feature   | 003 | Terminal Backend Adapter                  | [feature/003_terminal_backend_adapter.md](../feature/003_terminal_backend_adapter.md)                                |
| feature   | 004 | None Backend Adapter                      | [feature/004_none_backend_adapter.md](../feature/004_none_backend_adapter.md)                                        |
| feature   | 005 | WebGPU Backend Adapter                    | [feature/005_webgpu_backend_adapter.md](../feature/005_webgpu_backend_adapter.md)                                    |
| feature   | 006 | Native Backend Adapter                    | [feature/006_native_backend_adapter.md](../feature/006_native_backend_adapter.md)                                    |
| invariant | 001 | Y-Up Coordinate System                    | [invariant/001_y_up_coordinate_system.md](../invariant/001_y_up_coordinate_system.md)                                |
| invariant | 002 | SVG Injection-Safe Output                 | [invariant/002_svg_injection_safe_output.md](../invariant/002_svg_injection_safe_output.md)                          |
| invariant | 003 | Z-Layer Draw Ordering                     | [invariant/003_z_layer_draw_ordering.md](../invariant/003_z_layer_draw_ordering.md)                                  |
| invariant | 004 | Vector Representability of Commands       | [invariant/004_vector_representability_of_commands.md](../invariant/004_vector_representability_of_commands.md)      |
| pattern   | 001 | Ports and Adapters Backend Architecture   | [pattern/001_ports_and_adapters_backend_architecture.md](../pattern/001_ports_and_adapters_backend_architecture.md)  |
| pitfall   | 001 | ArrayBuffer Swap-Remove Buffer-Binding Violation | [pitfall/001_arraybuffer_swap_remove_buffer_binding_violation.md](../pitfall/001_arraybuffer_swap_remove_buffer_binding_violation.md) |
| pitfall   | 002 | GPU Instance Struct Field-Reorder Desync  | [pitfall/002_gpu_instance_struct_field_reorder_desync.md](../pitfall/002_gpu_instance_struct_field_reorder_desync.md) |

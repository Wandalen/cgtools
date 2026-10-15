# Pitfall Doc Definition

A **pitfall** documents one way this crate's API can be misused or misunderstood — the trap, why it happens, and how to avoid it. In `tilemap_renderer`, these are confirmed edge cases in GPU buffer handling that aren't obvious from reading the adapters alone, each recorded with its observable failure and mitigation. This collection holds one instance per known pitfall; the table below is the index into them.

### Scope

- **Purpose**: `tilemap_renderer`'s GPU buffer handling contains confirmed edge cases that are not obvious from reading the adapters alone.
- **Responsibility**: Document each confirmed trap, its observable failure, and its mitigation (or lack thereof).
- **In Scope**: Confirmed implementation pitfalls in the WebGL2 backend adapter.
- **Out of Scope**: Hypothetical or not-yet-observed failure modes; general WebGL2/SVG pitfalls not specific to this crate's design.

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [ArrayBuffer Swap-Remove Buffer-Binding Violation](001_arraybuffer_swap_remove_buffer_binding_violation.md) | WebGL2 spec forbids self-to-self GPU buffer copy; mitigated via a scratch buffer | ✅ |
| 002 | [GPU Instance Struct Field-Reorder Desync](002_gpu_instance_struct_field_reorder_desync.md) | Compile-time size/align assertions don't catch a same-size field reorder | ⚠️ |

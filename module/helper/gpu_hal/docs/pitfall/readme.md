# Pitfall Doc Definition

A **pitfall** documents one way this crate's API can be misused or misunderstood — the trap, why it happens, and how to avoid it. In `gpu_hal`, both confirmed pitfalls come from asymmetries across its four backends that aren't obvious from any single module's signature: which backends actually exist in a given build, and which backends can actually read pixels back. This collection holds one instance per known pitfall; the table below is the index into them.

### Scope

- **Purpose**: `gpu_hal`'s four-backend design has cross-backend asymmetries not obvious from reading any single module.
- **Responsibility**: Document confirmed traps, their observable failures, and mitigations.
- **In Scope**: Compile-time backend availability; browser-unsupported pixel readback.
- **Out of Scope**: WebGPU/WebGL/`wgpu`-spec-level gotchas not specific to this crate's own design.

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [Backend Availability Is Compile-Time, Not Runtime](001_backend_availability_compile_time_not_runtime.md) | An unavailable backend's constructor fails to compile, not to run | ✅ |
| 002 | [Pixel Readback Is Unsupported on Browser Backends](002_pixel_readback_native_only.md) | `Surface::pixels_read` type-checks everywhere but only succeeds on native/vulkan | ✅ |

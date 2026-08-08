# Pitfall Doc Entity

### Scope

- **Purpose**: `minwebgpu`'s WASM-exclusive design has a build-target trap not obvious from reading any single module.
- **Responsibility**: Document confirmed traps, their observable failures, and mitigations.
- **In Scope**: Native-target stub compilation behavior.
- **Out of Scope**: WebGPU-spec-level gotchas not specific to this crate's design.

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [Native-Target Builds Compile to a Non-Functional Stub](001_native_target_compiles_to_nonfunctional_stub.md) | `cargo check` succeeds off `wasm32` but every call errors at runtime | ✅ |

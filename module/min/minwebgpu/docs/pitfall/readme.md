# Pitfall Doc Definition

A **pitfall** documents one way this crate's API can be misused or misunderstood — the trap, why it happens, and how to avoid it. In `minwebgpu`, that trap comes from its WASM-exclusive design: native-target stub compilation behavior that isn't obvious from reading any single module, documented here with its observable failure and mitigation. This collection holds one instance per known pitfall; the table below is the index into them.

### Scope

- **Purpose**: `minwebgpu`'s WASM-exclusive design has a build-target trap not obvious from reading any single module.
- **Responsibility**: Document confirmed traps, their observable failures, and mitigations.
- **In Scope**: Native-target stub compilation behavior.
- **Out of Scope**: WebGPU-spec-level gotchas not specific to this crate's design.

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [Native-Target Builds Compile to a Non-Functional Stub](001_native_target_compiles_to_nonfunctional_stub.md) | `cargo check` succeeds off `wasm32` but every call errors at runtime | ✅ |

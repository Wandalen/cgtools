# Invariant Doc Definition

An **invariant** is a guarantee this crate enforces and callers may rely on. In `minwebgpu`, these are the safety and error-handling guarantees that hold across every module — the error handling contract, the unsafe-code prohibition, and the panic policy — stated once, explicitly, along with how each is enforced. This collection holds one instance per invariant, each pinned to where it is enforced in code; the table below is the index into them.

### Scope

- **Purpose**: The crate's safety and error-handling guarantees hold across every module and are worth stating once, explicitly.
- **Responsibility**: Document crate-wide invariants and their enforcement mechanisms.
- **In Scope**: Error handling contract, unsafe-code prohibition, panic policy.
- **Out of Scope**: Per-module error variant details (read `src/error.rs` directly).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [Result-Based Error Handling, No Unsafe Code](001_result_based_error_handling.md) | Every fallible op returns `Result<_, WebGPUError>`; zero `unsafe` blocks; no panics except internal bugs | ✅ |

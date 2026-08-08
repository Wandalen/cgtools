# Invariant Doc Entity

### Scope

- **Purpose**: The crate's safety and error-handling guarantees hold across every module and are worth stating once, explicitly.
- **Responsibility**: Document crate-wide invariants and their enforcement mechanisms.
- **In Scope**: Error handling contract, unsafe-code prohibition, panic policy.
- **Out of Scope**: Per-module error variant details (read `src/error.rs` directly).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [Result-Based Error Handling, No Unsafe Code](001_result_based_error_handling.md) | Every fallible op returns `Result<_, WebGPUError>`; zero `unsafe` blocks; no panics except internal bugs | ✅ |

# Pitfall Doc Definition

A **pitfall** documents one way this crate's API can be misused or misunderstood — the trap, why it happens, and how to avoid it. In `tilemap_scene`, that's the load-time validation gaps that let malformed specs pass silently, each recorded with its concrete failure mode and mitigation. This collection holds one instance per known pitfall; the table below is the index into them.

### Scope

- **Purpose**: Navigational hub for `tilemap_scene`'s known traps — non-obvious ways to get bitten by the current implementation.
- **Responsibility**: Document each trap, its concrete failure mode, and mitigation.
- **In Scope**: Load-time validation gaps that let malformed specs pass silently.
- **Out of Scope**: The full validation-rule breakdown these pitfalls summarize (see `invariant/`).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [Load-Time Validation Is Only Partially Enforced](001_load_time_validation_partially_enforced.md) | A successful `load()` is not proof of renderability | ✅ |

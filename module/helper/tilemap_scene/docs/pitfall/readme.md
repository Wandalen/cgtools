# Pitfall Doc Definition

### Scope

- **Purpose**: Navigational hub for `tilemap_scene`'s known traps — non-obvious ways to get bitten by the current implementation.
- **Responsibility**: Document each trap, its concrete failure mode, and mitigation.
- **In Scope**: Load-time validation gaps that let malformed specs pass silently.
- **Out of Scope**: The full validation-rule breakdown these pitfalls summarize (see `invariant/`).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [Load-Time Validation Is Only Partially Enforced](001_load_time_validation_partially_enforced.md) | A successful `load()` is not proof of renderability | ✅ |

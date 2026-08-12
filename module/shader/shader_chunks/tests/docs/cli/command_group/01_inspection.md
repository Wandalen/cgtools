# Command Group Test :: Inspection

Source: [`../../../../docs/cli/command_group.md`](../../../../docs/cli/command_group.md)

### Group Cases (CG-N)

| ID | Invariant | Evidence |
|----|-----------|----------|
| CG-1 | No member command produces a side effect outside stdout content and process exit code — an error path exits non-zero without a panic backtrace | `cli_subprocess_test.rs::get_unknown_chunk_exits_non_zero_without_a_panic_backtrace`; `cli_subprocess_test.rs::compose_missing_dependency_exits_non_zero_without_a_panic_backtrace` |
| CG-2 | Every member command operates only on the compiled-in `shader_chunks_core::ALL_CHUNKS` registry — no filesystem or environment access | Structural: `src/lib.rs` and `src/main.rs` contain no `std::fs`/`std::env` usage (confirmed by direct source read during doc authoring; mechanically re-checkable via `grep -rn "std::fs\|std::env" src/`) |
| CG-3 | Idempotent — every command's output is a pure function of its arguments, no mutable state | Structural: no `&mut` receiver or interior mutability anywhere in `src/lib.rs`'s public functions (`list_chunks`, `get_chunk`, `list_tags`, `tree_chunk`, `try_compose_wgsl`, `compose_chunks` all take `&str`/`&[&str]`/`Option<&str>` and return a fresh `Result<String, CliError>`) |

CG-2 and CG-3 are structural invariants rather than behavior a unit test
observes directly — there is no dedicated "call twice, compare" test in
`shader_chunks_test.rs`, and adding one would test the Rust type
system's own purity guarantees rather than this crate's logic. Documented
here as a structural check per `§ Test Coverage Summary` below, honestly
distinguished from CG-1's genuine behavioral test evidence.

### Membership Coverage

Confirms the group's Semantic Coherence Test
("[`command_group.md`](../../../../docs/cli/command_group.md#semantic-coherence-test)")
holds for every current member:

| Command | Answers | Confirmed |
|---------|---------|-----------|
| `.list` | What chunks exist | ✅ |
| `.get` | What does this one chunk look like | ✅ |
| `.tags` | What tags exist and on which chunks | ✅ |
| `.tree` | What does this chunk depend on | ✅ |
| `.compose` | What would composing these chunks produce | ✅ |

### Test Coverage Summary

| Metric | Value |
|--------|-------|
| Group cases | 3 |
| Behaviorally tested | 1 (CG-1) |
| Structurally verified | 2 (CG-2, CG-3) |
| Membership coverage | 5/5 commands |

### Cross-References

| File | Relationship |
|------|--------------|
| [`../command/readme.md`](../command/readme.md) | Member command test specs |
| [`../../../../docs/cli/command_group.md`](../../../../docs/cli/command_group.md) | Group documentation source |

# CLI Test Specifications

### Scope

- **Purpose:** Test specifications for this crate's 1 command, organized
  by testing lens (command, command group) rather than by test file.
- **Responsibility:** Cross-reference every documented CLI entity in
  [`../../docs/cli/`](../../docs/cli/readme.md) to the real test
  function(s) that verify it.
- **In Scope:** The 2 tiers below, covering this crate's 1 command and 1
  command group.
- **Out of Scope:** Test implementation itself (→ `shader_chunks_compose/tests/shader_chunks_compose_test.rs`)
  plus the aggregator's own subprocess suite (→
  [`../../../../shader_chunks/tests/cli_subprocess_test.rs`](../../../../shader_chunks/tests/cli_subprocess_test.rs));
  CLI behavior documentation (→ [`../../docs/cli/readme.md`](../../docs/cli/readme.md));
  parameter-level tiers, owned by `shader_chunks_query` (→
  [`../../../../shader_chunks_query/tests/docs/cli/readme.md`](../../../../shader_chunks_query/tests/docs/cli/readme.md));
  the other 7 commands of the `shader_chunks` family (→
  [family index](../../../../shader_chunks/tests/docs/cli/readme.md)).

---

### Architecture

This tree mirrors [`../../docs/cli/`](../../docs/cli/readme.md)'s own
entity structure in a parallel tree, per `cli_doc_des.rulebook.md §
Parameters Documentation : Testing Directory Structure · OC118` and `§
Directory Authority : DIR-01`. This crate declares only 2 of the
family's 5 tiers — it has no `param/`, `param_group/`, or `type/` of its
own (`compose`'s parameters are owned by `shader_chunks_query`), and
`format/` has no test-doc mirror anywhere in the family:

| Tier | Directory | Lens | Prefix |
|------|-----------|------|--------|
| 3 | [`command/`](command/readme.md) | Per-command integration | `INT-N` / `PAR-N` |
| 4 | [`command_group/`](command_group/readme.md) | Cross-command group invariants + workflows | `CG-N` / `WF-N` |

### Aggregate Test Counts

| Tier | Files | Test Cases |
|------|-------|------------|
| command/ | 1 | 5 `PAR-N` + 0 `INT-N` |
| command_group/ | 1 | 5 `CG-N` + 1 `WF-N` |

Counts overlap by design (Overlap Policy, OC118) — the same real test
function is cited from multiple tiers when it verifies more than one
entity. `compose`'s own `command/` file has no standalone `INT-N` rows —
its cross-group workflow case is filed under `command_group/`'s WF-1
instead (`tree` then `compose`).

### Test Category Definitions

- **`PAR-N`** — Parameter test as exercised by `compose`.
- **`INT-N`** — Integration: an end-to-end command invocation.
- **`CG-N`** — Command Group: an invariant shared by every member of a
  functional cluster.
- **`WF-N`** — Workflow: a documented multi-command usage pattern,
  verified by composing each step's own test (valid because every
  command is stateless and idempotent).

### Usage Guide

- **Implementers** — read `shader_chunks_query/tests/docs/cli/param/`
  and `type/` for the exact validation and parsing contract `names`/
  `transitive` must satisfy.
- **Testers** — read `command/` and `command_group/` for end-to-end
  scenarios and cross-command invariants.
- **Coverage trackers** — cross-reference this tree's Real Test columns
  against `shader_chunks_compose/tests/shader_chunks_compose_test.rs`
  (see Out of Scope above) and
  [`../../../../shader_chunks/tests/cli_subprocess_test.rs`](../../../../shader_chunks/tests/cli_subprocess_test.rs)
  to confirm no cited function is missing.

### Test Priority Levels

- **P0** — Exit-code-affecting behavior (unknown chunk, missing
  dependency, cyclic dependency).
- **P1** — Structural output correctness (dependency-ordered WGSL text).
- **P2** — Help behavior (top-level grouping).

### Navigation

- [`command/`](command/readme.md) — Tier 3
- [`command_group/`](command_group/readme.md) — Tier 4
- [`../../docs/cli/readme.md`](../../docs/cli/readme.md) — documentation source root (this crate)
- [`../../../../shader_chunks/tests/docs/cli/readme.md`](../../../../shader_chunks/tests/docs/cli/readme.md) — family-wide test-doc index

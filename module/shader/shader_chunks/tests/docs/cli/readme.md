# CLI Test Specifications

### Scope

- **Purpose:** Test specifications for `shader_chunks`, organized by testing lens (parameter, parameter group, command, command group, type) rather than by test file.
- **Responsibility:** Cross-reference every documented CLI entity in `../../../docs/cli/` to the real test function(s) that verify it.
- **In Scope:** The 5 tiers below, covering all 6 commands, 21 parameters, 3 parameter groups, 4 command groups, and 10 types this CLI declares.
- **Out of Scope:** Test implementation itself (→ [`../../shader_chunks_test.rs`](../../shader_chunks_test.rs), [`../../cli_subprocess_test.rs`](../../cli_subprocess_test.rs)); CLI behavior documentation (→ [`../../../docs/cli/readme.md`](../../../docs/cli/readme.md)).

---

### Architecture

This CLI mirrors `docs/cli/`'s own entity structure in a parallel tree, per
`cli_doc_des.rulebook.md § Parameters Documentation : Testing Directory
Structure · OC118` and `§ Directory Authority : DIR-01`. All 5 tiers are
populated:

| Tier | Directory | Lens | Prefix |
|------|-----------|------|--------|
| 1 | [`param/`](param/readme.md) | Parameter edge cases | `EC-N` |
| 2 | [`param_group/`](param_group/readme.md) | Group-interaction corner cases | `GRP-N` |
| 3 | [`command/`](command/readme.md) | Per-command integration | `INT-N` / `PAR-N` |
| 4 | [`command_group/`](command_group/readme.md) | Cross-command group invariants + workflows | `CG-N` / `WF-N` |
| 5 | [`type/`](type/readme.md) | Type construction/parsing/rejection | `TC-N` |

Tier 2 was formerly omitted (the pre-unification CLI had no co-occurring
parameter set); the shared 19-named-parameter query surface across
`list`/`get` now makes it mandatory.

### Aggregate Test Counts

| Tier | Files | Test Cases |
|------|-------|------------|
| param/ | 21 | 71 `EC-N` |
| param_group/ | 3 | 15 `GRP-N` |
| command/ | 6 | 19 `PAR-N` + 9 `INT-N` |
| command_group/ | 4 | 19 `CG-N` + 3 `WF-N` |
| type/ | 10 | 33 `TC-N` |

Counts overlap by design (Overlap Policy, OC118) — the same real test
function is cited from multiple tiers when it verifies more than one
entity (e.g. `query_enum_params_round_trip_and_reject_bogus_values`
backs 4 type mirrors and 4 parameter mirrors).

### Test Category Definitions

- **`EC-N`** — Edge Case: a single parameter's boundary condition (empty,
  unknown, valid).
- **`GRP-N`** — Group corner: an interaction rule between members of one
  parameter group (conjunctive filters, short-circuits, no-ops).
- **`PAR-N`** — Parameter test as exercised by one specific command.
- **`INT-N`** — Integration: an end-to-end command invocation.
- **`CG-N`** — Command Group: an invariant shared by every member of a
  functional cluster.
- **`WF-N`** — Workflow: a documented multi-command usage pattern,
  verified by composing each step's own test (valid because every
  command is stateless and idempotent).
- **`TC-N`** — Type Case: construction, parsing, or rejection for one CLI
  type.

### Usage Guide

- **Implementers** — read `param/` and `type/` for the exact validation
  and parsing contract each parameter/type must satisfy.
- **Testers** — read `param_group/`, `command/`, and `command_group/` for
  interaction rules, end-to-end scenarios, and cross-command invariants.
- **Coverage trackers** — cross-reference this tree's Real Test columns
  against `../../shader_chunks_test.rs` and
  `../../cli_subprocess_test.rs` to confirm no cited function is missing.

### Test Priority Levels

- **P0** — Exit-code-affecting behavior (unknown chunk/field, invalid
  enum or integer values, missing required `names`, missing dependency,
  cyclic dependency).
- **P1** — Structural output correctness (table columns, format shapes,
  sort/page determinism, tree ordering, compose ordering).
- **P2** — Help behavior (top-level grouping, per-command defaults,
  no-argument fallback).

### Navigation

- [`param/`](param/readme.md) — Tier 1
- [`param_group/`](param_group/readme.md) — Tier 2
- [`command/`](command/readme.md) — Tier 3
- [`command_group/`](command_group/readme.md) — Tier 4
- [`type/`](type/readme.md) — Tier 5

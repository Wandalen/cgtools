# CLI Test Specifications

### Scope

- **Purpose:** Test specifications for `shader_chunks`, organized by testing lens (parameter, parameter group, command, command group, type) rather than by test file.
- **Responsibility:** Cross-reference every documented CLI entity in `../../../docs/cli/` to the real test function(s) that verify it.
- **In Scope:** The 5 tiers below, covering all 8 commands, 26 parameters, 3 parameter groups, 6 command groups, and 11 types this CLI declares.
- **Out of Scope:** Test implementation itself — now split across each utility's own crate (→ `shader_chunks_query_core/tests/shader_chunks_query_core_test.rs`, `shader_chunks_compose/tests/shader_chunks_compose_test.rs`, `shader_chunks_params/tests/tunables_test.rs`, `shader_chunks_preview/tests/preview_cli_test.rs`, `shader_chunks_render/tests/render_cli_test.rs` with its engine tier in `shader_chunks_render_core/tests/render_core_test.rs`) plus the aggregator's own subprocess suite (→ [`../../cli_subprocess_test.rs`](../../cli_subprocess_test.rs)); CLI behavior documentation (→ [`../../../docs/cli/readme.md`](../../../docs/cli/readme.md)).

---

### Architecture

This CLI mirrors `docs/cli/`'s own entity structure in a parallel tree, per
`cli_doc_des.rulebook.md § Parameters Documentation : Testing Directory
Structure · OC118` and `§ Directory Authority : DIR-01`. Per the family's
leaf-locality split (see
[`../../../docs/cli/readme.md#scope-decisions`](../../../docs/cli/readme.md#scope-decisions)),
each of the 5 tiers is now realized independently inside whichever leaf
crate(s) actually own that entity class — not every leaf has every tier,
since not every leaf declares params/param_groups/types of its own:

| Tier | Lens | Prefix | Realized in |
|------|------|--------|-------------|
| 1 | Parameter edge cases | `EC-N` | query, preview, render |
| 2 | Group-interaction corner cases | `GRP-N` | query only |
| 3 | Per-command integration | `INT-N` / `PAR-N` | all 5 leaves |
| 4 | Cross-command group invariants + workflows | `CG-N` / `WF-N` | all 5 leaves |
| 5 | Type construction/parsing/rejection | `TC-N` | query, render |

Tier 2 exists only for `shader_chunks_query`: the shared 19-named-parameter
query surface across `list`/`get` is the sole co-occurring parameter set
anywhere in the family — no other leaf's parameters form a group.

### Per-Leaf Tier Coverage

| Crate | Tier 1 `param/` | Tier 2 `param_group/` | Tier 3 `command/` | Tier 4 `command_group/` | Tier 5 `type/` |
|-------|------------------|------------------------|---------------------|---------------------------|------------------|
| [`shader_chunks_query`](../../../../shader_chunks_query/tests/docs/cli/readme.md) | [param/](../../../../shader_chunks_query/tests/docs/cli/param/readme.md) | [param_group/](../../../../shader_chunks_query/tests/docs/cli/param_group/readme.md) | [command/](../../../../shader_chunks_query/tests/docs/cli/command/readme.md) | [command_group/](../../../../shader_chunks_query/tests/docs/cli/command_group/readme.md) | [type/](../../../../shader_chunks_query/tests/docs/cli/type/readme.md) |
| [`shader_chunks_compose`](../../../../shader_chunks_compose/tests/docs/cli/readme.md) | — | — | [command/](../../../../shader_chunks_compose/tests/docs/cli/command/readme.md) | [command_group/](../../../../shader_chunks_compose/tests/docs/cli/command_group/readme.md) | — |
| [`shader_chunks_params`](../../../../shader_chunks_params/tests/docs/cli/readme.md) | — | — | [command/](../../../../shader_chunks_params/tests/docs/cli/command/readme.md) | [command_group/](../../../../shader_chunks_params/tests/docs/cli/command_group/readme.md) | — |
| [`shader_chunks_preview`](../../../../shader_chunks_preview/tests/docs/cli/readme.md) | [param/](../../../../shader_chunks_preview/tests/docs/cli/param/readme.md) | — | [command/](../../../../shader_chunks_preview/tests/docs/cli/command/readme.md) | [command_group/](../../../../shader_chunks_preview/tests/docs/cli/command_group/readme.md) | — |
| [`shader_chunks_render`](../../../../shader_chunks_render/tests/docs/cli/readme.md) | [param/](../../../../shader_chunks_render/tests/docs/cli/param/readme.md) | — | [command/](../../../../shader_chunks_render/tests/docs/cli/command/readme.md) | [command_group/](../../../../shader_chunks_render/tests/docs/cli/command_group/readme.md) | — |

### Aggregate Test Counts

Family-wide totals — stable regardless of which leaf owns which file.
Per-file breakdown now lives in each leaf's own Aggregate Test Counts
table rather than being duplicated here:

| Tier | Files | Test Cases | Per-leaf breakdown |
|------|-------|------------|---------------------|
| param/ | 26 | 98 `EC-N` | [query](../../../../shader_chunks_query/tests/docs/cli/readme.md#aggregate-test-counts) · [preview](../../../../shader_chunks_preview/tests/docs/cli/readme.md#aggregate-test-counts) · [render](../../../../shader_chunks_render/tests/docs/cli/readme.md#aggregate-test-counts) |
| param_group/ | 3 | 15 `GRP-N` | [query](../../../../shader_chunks_query/tests/docs/cli/readme.md#aggregate-test-counts) |
| command/ | 8 | 32 `PAR-N` + 27 `INT-N` | all 5 leaves, see [Per-Leaf Tier Coverage](#per-leaf-tier-coverage) above |
| command_group/ | 6 | 32 `CG-N` + 3 `WF-N` | all 5 leaves, see [Per-Leaf Tier Coverage](#per-leaf-tier-coverage) above |
| type/ | 11 | 40 `TC-N` | [query](../../../../shader_chunks_query/tests/docs/cli/readme.md#aggregate-test-counts) · [render](../../../../shader_chunks_render/tests/docs/cli/readme.md#aggregate-test-counts) |

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
  against each cited crate's own `tests/*.rs` file (see Out of Scope
  above) and [`../../cli_subprocess_test.rs`](../../cli_subprocess_test.rs)
  to confirm no cited function is missing.

### Test Priority Levels

- **P0** — Exit-code-affecting behavior (unknown chunk/field, invalid
  enum or integer values, missing required `names`, missing dependency,
  cyclic dependency).
- **P1** — Structural output correctness (table columns, format shapes,
  sort/page determinism, tree ordering, compose ordering).
- **P2** — Help behavior (top-level grouping, per-command defaults,
  no-argument fallback).

### Navigation

- [`shader_chunks_query/tests/docs/cli/`](../../../../shader_chunks_query/tests/docs/cli/readme.md) — Tiers 1-5 (`list`, `get`, `tags`, `tree`)
- [`shader_chunks_compose/tests/docs/cli/`](../../../../shader_chunks_compose/tests/docs/cli/readme.md) — Tiers 3-4 (`compose`)
- [`shader_chunks_params/tests/docs/cli/`](../../../../shader_chunks_params/tests/docs/cli/readme.md) — Tiers 3-4 (`tunables`)
- [`shader_chunks_preview/tests/docs/cli/`](../../../../shader_chunks_preview/tests/docs/cli/readme.md) — Tiers 1, 3-4 (`preview`)
- [`shader_chunks_render/tests/docs/cli/`](../../../../shader_chunks_render/tests/docs/cli/readme.md) — Tiers 1, 3-5 (`render`)
- [`../../../docs/cli/readme.md`](../../../docs/cli/readme.md) — CLI behavior documentation family index

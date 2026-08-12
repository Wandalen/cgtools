# CLI Test Specifications

### Scope

- **Purpose:** Test specifications for `shader_chunks_cli`, organized by testing lens (parameter, command, command group, type) rather than by test file.
- **Responsibility:** Cross-reference every documented CLI entity in `../../../docs/cli/` to the real test function(s) that verify it.
- **In Scope:** The 4 active tiers below, covering all 5 commands, 2 parameters, 1 command group, and 1 type this CLI declares.
- **Out of Scope:** Test implementation itself (→ [`../../shader_chunks_cli_test.rs`](../../shader_chunks_cli_test.rs), [`../../cli_subprocess_test.rs`](../../cli_subprocess_test.rs)); CLI behavior documentation (→ [`../../../docs/cli/readme.md`](../../../docs/cli/readme.md)).

---

### Architecture

This CLI mirrors `docs/cli/`'s own entity structure in a parallel tree, per
`cli_doc_des.rulebook.md § Parameters Documentation : Testing Directory
Structure · OC118` and `§ Directory Authority : DIR-01`. The rulebook
defines 5 possible tiers; this CLI populates 4 of them:

| Tier | Directory | Lens | Prefix |
|------|-----------|------|--------|
| 1 | [`param/`](param/readme.md) | Parameter edge cases | `EC-N` |
| 3 | [`command/`](command/readme.md) | Command integration + per-command | `INT-N` / `PAR-N` |
| 4 | [`command_group/`](command_group/readme.md) | Cross-command group invariants | `CG-N` |
| 5 | [`type/`](type/readme.md) | Type construction/parsing/rejection | `TC-N` |

**Tier 2 (`param_group/`) is omitted** — this CLI declares zero parameter
groups (no two commands share a co-occurring parameter *set*); see
[`../../../docs/cli/readme.md` § Scope
Decisions](../../../docs/cli/readme.md#scope-decisions). `GRP-N` test
cases therefore do not exist in this tree.

### Aggregate Test Counts

| Tier | Files | Real Test Functions Referenced |
|------|-------|----------------------------------|
| param/ | 2 | 11 (from `shader_chunks_cli_test.rs`) |
| command/ | 6 (1 category + 5 per-command) | 12 (from both test files) |
| command_group/ | 1 | 12 (from both test files, aggregated) |
| type/ | 1 | 11 (from `shader_chunks_cli_test.rs`) |

Counts overlap by design (Overlap Policy, OC118) — the same real test
function is cited from multiple tiers when it verifies more than one
entity (e.g. a `get`-command test also verifies the `name` parameter and
the `ChunkName` type).

### Test Category Definitions

- **`EC-N`** — Edge Case: a single parameter's boundary condition (empty,
  unknown, valid).
- **`PAR-N`** — Parameter test as exercised by one specific command.
- **`INT-N`** — Integration: an end-to-end command invocation or
  cross-command workflow.
- **`CG-N`** — Command Group: an invariant shared by every member of a
  functional cluster.
- **`TC-N`** — Type Case: construction, parsing, or rejection for one CLI
  type.

### Usage Guide

- **Implementers** — read `param/` and `type/` for the exact validation
  and parsing contract each parameter/type must satisfy.
- **Testers** — read `command/` and `command_group/` for end-to-end
  scenarios and cross-command invariants.
- **Coverage trackers** — cross-reference this tree's Real Test columns
  against `../../shader_chunks_cli_test.rs` and
  `../../cli_subprocess_test.rs` to confirm no cited function is missing.

### Test Priority Levels

- **P0** — Exit-code-affecting behavior (unknown chunk, missing
  dependency, cyclic dependency) — covered by every tier.
- **P1** — Structural output correctness (table columns, tree ordering,
  compose ordering) — covered by `command/` and `type/`.
- **P2** — Help/no-argument fallback — covered by `command/`'s category
  file only.

### Navigation

- [`param/`](param/readme.md) — Tier 1
- [`command/`](command/readme.md) — Tier 3
- [`command_group/`](command_group/readme.md) — Tier 4
- [`type/`](type/readme.md) — Tier 5

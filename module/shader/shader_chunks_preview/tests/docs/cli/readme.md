# CLI Test Specifications

### Scope

- **Purpose:** Test specifications for this crate's 1 command, organized
  by testing lens (parameter, command, command group) rather than by
  test file.
- **Responsibility:** Cross-reference every documented CLI entity in
  [`../../docs/cli/`](../../docs/cli/readme.md) to the real test
  function(s) that verify it.
- **In Scope:** The 3 tiers below, covering this crate's 1 command, 2 own
  parameters, and 1 command group.
- **Out of Scope:** Test implementation itself (→ `shader_chunks_preview/tests/preview_cli_test.rs`,
  engine tier → `shader_chunks_preview_core`) plus the aggregator's own
  subprocess suite (→
  [`../../../../shader_chunks/tests/cli_subprocess_test.rs`](../../../../shader_chunks/tests/cli_subprocess_test.rs));
  CLI behavior documentation (→ [`../../docs/cli/readme.md`](../../docs/cli/readme.md));
  the other 4 leaf CLIs' commands (→
  [family index](../../../../shader_chunks/tests/docs/cli/readme.md)).

---

### Architecture

This tree mirrors [`../../docs/cli/`](../../docs/cli/readme.md)'s own
entity structure in a parallel tree, per `cli_doc_des.rulebook.md §
Parameters Documentation : Testing Directory Structure · OC118` and `§
Directory Authority : DIR-01`. This crate declares no `param_group/` or
`type/` tier (see [`../../docs/cli/readme.md`](../../docs/cli/readme.md)'s
Completion Matrix note) and `format/` has no mirror anywhere in the
family:

| Tier | Directory | Lens | Prefix |
|------|-----------|------|--------|
| 1 | [`param/`](param/readme.md) | Parameter edge cases | `EC-N` |
| 3 | [`command/`](command/readme.md) | Per-command integration | `INT-N` / `PAR-N` |
| 4 | [`command_group/`](command_group/readme.md) | Cross-command group invariants + workflows | `CG-N` / `WF-N` |

### Aggregate Test Counts

| Tier | Files | Test Cases |
|------|-------|------------|
| param/ | 2 | 12 `EC-N` |
| command/ | 1 | 5 `PAR-N` + 6 `INT-N` |
| command_group/ | 1 | 6 `CG-N` + 0 `WF-N` |

Counts overlap by design (Overlap Policy, OC118) — the same real test
function is cited from multiple tiers when it verifies more than one
entity. The 0 `WF-N` here is a disclosed gap, not an omission: no test
composes a preview-then-render (or similar) real-test pair today — see
[`command_group/01_preview.md`](command_group/01_preview.md)'s Workflow
Compositions section.

### Test Category Definitions

- **`EC-N`** — Edge Case: a single parameter's boundary condition (empty,
  unknown, valid).
- **`PAR-N`** — Parameter test as exercised by one specific command.
- **`INT-N`** — Integration: an end-to-end command invocation.
- **`CG-N`** — Command Group: an invariant shared by every member of a
  functional cluster.
- **`WF-N`** — Workflow: a documented multi-command usage pattern,
  verified by composing each step's own test.

### Usage Guide

- **Implementers** — read `param/` for the exact validation contract
  each parameter must satisfy.
- **Testers** — read `command/` and `command_group/` for end-to-end
  scenarios and cross-command invariants.
- **Coverage trackers** — cross-reference this tree's Real Test columns
  against `shader_chunks_preview/tests/preview_cli_test.rs` (see Out of
  Scope above) and
  [`../../../../shader_chunks/tests/cli_subprocess_test.rs`](../../../../shader_chunks/tests/cli_subprocess_test.rs)
  to confirm no cited function is missing.

### Test Priority Levels

- **P0** — Exit-code-affecting behavior (unknown chunk, missing/both
  targets, unreadable `file::`, naga validation failure).
- **P1** — Structural output correctness (summary content, slider
  listing).
- **P2** — Help behavior (top-level grouping, no-argument fallback).

### Navigation

- [`param/`](param/readme.md) — Tier 1
- [`command/`](command/readme.md) — Tier 3
- [`command_group/`](command_group/readme.md) — Tier 4
- [`../../docs/cli/readme.md`](../../docs/cli/readme.md) — documentation source root (this crate)
- [`../../../../shader_chunks/tests/docs/cli/readme.md`](../../../../shader_chunks/tests/docs/cli/readme.md) — family-wide test-doc index

# CLI Test Specifications

### Scope

- **Purpose:** Test specifications for this crate's 1 command, organized
  by testing lens (parameter, command, command group, type) rather than
  by test file.
- **Responsibility:** Cross-reference every documented CLI entity in
  [`../../docs/cli/`](../../docs/cli/readme.md) to the real test
  function(s) that verify it.
- **In Scope:** The 4 tiers below, covering this crate's 1 command, 4
  owned parameters, 1 command group, and 2 owned types.
- **Out of Scope:** Test implementation itself (→ `../../render_cli_test.rs`
  plus the engine tier in
  `../../../shader_chunks_render_core/tests/render_core_test.rs`) and
  the aggregator's own subprocess suite (→
  [`../../../../shader_chunks/tests/cli_subprocess_test.rs`](../../../../shader_chunks/tests/cli_subprocess_test.rs));
  CLI behavior documentation (→ [`../../docs/cli/readme.md`](../../docs/cli/readme.md));
  the other 4 commands' worth of tiers in the `shader_chunks` family (→
  [family index](../../../../shader_chunks/tests/docs/cli/readme.md)).

---

### Architecture

This tree mirrors [`../../docs/cli/`](../../docs/cli/readme.md)'s own
entity structure in a parallel tree, per `cli_doc_des.rulebook.md §
Parameters Documentation : Testing Directory Structure · OC118` and `§
Directory Authority : DIR-01`. This crate has no `param_group/` tier —
`render`'s parameters belong to no co-occurrence group (see
[`../../docs/cli/param/readme.md`](../../docs/cli/param/readme.md)) — and
`format/` has no mirror anywhere in the family:

| Tier | Directory | Lens | Prefix |
|------|-----------|------|--------|
| 1 | [`param/`](param/readme.md) | Parameter edge cases | `EC-N` |
| 3 | [`command/`](command/readme.md) | Per-command integration | `INT-N` / `PAR-N` |
| 4 | [`command_group/`](command_group/readme.md) | Cross-command group invariants + workflows | `CG-N` / `WF-N` |
| 5 | [`type/`](type/readme.md) | Type construction/parsing/rejection | `TC-N` |

### Aggregate Test Counts

| Tier | Files | Test Cases |
|------|-------|------------|
| param/ | 4 | 24 `EC-N` |
| command/ | 1 | 13 `PAR-N` + 15 `INT-N` |
| command_group/ | 1 | 7 `CG-N` + 0 `WF-N` |
| type/ | 2 | 10 `TC-N` |

Counts overlap by design (Overlap Policy, OC118) — the same real test
function is cited from multiple tiers when it verifies more than one
entity. The 0 `WF-N` is a disclosed gap, not an omission: `render`'s
group doc describes `preview` then `render` as the natural sequence, but
no test composes a preview-then-render real-test pair — see
[`command_group/01_render.md`](command_group/01_render.md).

### Test Category Definitions

- **`EC-N`** — Edge Case: a single parameter's boundary condition (empty,
  unknown, valid).
- **`PAR-N`** — Parameter test as exercised by one specific command.
- **`INT-N`** — Integration: an end-to-end command invocation.
- **`CG-N`** — Command Group: an invariant shared by every member of a
  functional cluster.
- **`WF-N`** — Workflow: a documented multi-command usage pattern,
  verified by composing each step's own test.
- **`TC-N`** — Type Case: construction, parsing, or rejection for one CLI
  type.

### Usage Guide

- **Implementers** — read `param/` and `type/` for the exact validation
  and parsing contract each parameter/type must satisfy.
- **Testers** — read `command/` and `command_group/` for end-to-end
  scenarios and cross-command invariants.
- **Coverage trackers** — cross-reference this tree's Real Test columns
  against `../../render_cli_test.rs`,
  `../../../shader_chunks_render_core/tests/render_core_test.rs` (see Out
  of Scope above), and
  [`../../../../shader_chunks/tests/cli_subprocess_test.rs`](../../../../shader_chunks/tests/cli_subprocess_test.rs)
  to confirm no cited function is missing.

### Test Priority Levels

- **P0** — Exit-code-affecting behavior (unknown chunk, unreadable/missing
  file target, both/neither target given, invalid `size`/`time`).
- **P1** — Structural output correctness (PNG decodes at the requested
  size, summary line content).
- **P2** — Help behavior (group rendering, no-argument fallback).

### Navigation

- [`param/`](param/readme.md) — Tier 1
- [`command/`](command/readme.md) — Tier 3
- [`command_group/`](command_group/readme.md) — Tier 4
- [`type/`](type/readme.md) — Tier 5
- [`../../docs/cli/readme.md`](../../docs/cli/readme.md) — documentation source root (this crate)
- [`../../../../shader_chunks/tests/docs/cli/readme.md`](../../../../shader_chunks/tests/docs/cli/readme.md) — family-wide test-doc index

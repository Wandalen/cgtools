# Commands

### Scope

- **Purpose:** Documents every command this crate exposes.
- **Responsibility:** One dedicated file per command with full syntax,
  parameters, examples, and cross-references.
- **In Scope:** The 1 command dispatched by this crate's contribution to
  `shader_chunks_cli_core`'s `CommandRegistry` — `validate`.
- **Out of Scope:** Output rendering detail — its format (`plain_text`)
  is owned by
  [`shader_chunks_compose`](../../../../shader_chunks_compose/docs/cli/format/01_plain_text.md);
  the other 8 commands of the `shader_chunks` family (→
  [family index](../../../../shader_chunks/docs/cli/readme.md)).

---

### Overview Table

| # | File | Command | Params | Status |
|---|------|---------|--------|--------|
| 1 | [01_validate.md](01_validate.md) | `.validate` | 0 | ✅ |

**Total:** 1 command (of 9 across the `shader_chunks` family)

### Commands Table

| # | Command | Purpose | Params | Group |
|---|---------|---------|--------|-------|
| 1 | `.validate` | Lint the bundled registry: drift, duplicates, missing/cyclic deps, WGSL compile | 0 | [Validate](../command_group/01_validate.md) |

This crate's sole command is a single-member partition of the family's 7
[command groups](../command_group/readme.md) — see
[`01_validate.md`](../command_group/01_validate.md#why-not-merge-into-query-compose-preview-render)
for why it is not folded into an existing group despite consulting only
the compiled-in registry, same as Query. The remaining 8 commands of the
`shader_chunks` family live in their own crates — see the
[family index](../../../../shader_chunks/docs/cli/readme.md).

### Docs

| File | Relationship |
|------|--------------|
| [../readme.md](../readme.md) | CLI documentation root (this crate) |
| [../command_group/readme.md](../command_group/readme.md) | Command groups partitioning this layer |
| [`plain_text` format](../../../../shader_chunks_compose/docs/cli/format/01_plain_text.md) | Output format definition (owned by `shader_chunks_compose`) |

### Tests

| File | Relationship |
|------|--------------|
| [../../../tests/docs/cli/command/readme.md](../../../tests/docs/cli/command/readme.md) | Command-level test specifications |

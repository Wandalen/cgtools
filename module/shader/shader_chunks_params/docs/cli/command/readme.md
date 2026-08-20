# Commands

### Scope

- **Purpose:** Documents every command this crate exposes.
- **Responsibility:** One dedicated file per command with full syntax,
  parameters, examples, and cross-references.
- **In Scope:** The 1 command dispatched by this crate's contribution to
  `shader_chunks_cli_core`'s `CommandRegistry` — `tunables`.
- **Out of Scope:** Parameter type/constraint detail — this command's
  sole parameter (`name`) is owned by
  [`shader_chunks_query`](../../../../shader_chunks_query/docs/cli/param/01_name.md);
  output rendering detail — its format (`table_plain`) is likewise owned
  by [`shader_chunks_query`](../../../../shader_chunks_query/docs/cli/format/01_table_plain.md);
  the other 7 commands of the `shader_chunks` family (→
  [family index](../../../../shader_chunks/docs/cli/readme.md)).

---

### Overview Table

| # | File | Command | Params | Status |
|---|------|---------|--------|--------|
| 1 | [01_tunables.md](01_tunables.md) | `.tunables` | 1 | ✅ |

**Total:** 1 command (of 9 across the `shader_chunks` family)

### Commands Table

| # | Command | Purpose | Params | Group |
|---|---------|---------|--------|-------|
| 1 | `.tunables` | List a chunk's declared tunable parameters | 1 | [Parameters](../command_group/01_parameters.md) |

This crate's sole command is a single-member partition of the family's 6
[command groups](../command_group/readme.md) — see
[`01_parameters.md`](../command_group/01_parameters.md#why-not-merge-into-query)
for why it is not folded into the Query group despite sharing the same
positional-selector shape. The remaining 7 commands of the
`shader_chunks` family live in their own crates — see the
[family index](../../../../shader_chunks/docs/cli/readme.md).

### Docs

| File | Relationship |
|------|--------------|
| [../readme.md](../readme.md) | CLI documentation root (this crate) |
| [../command_group/readme.md](../command_group/readme.md) | Command groups partitioning this layer |
| [`name` param](../../../../shader_chunks_query/docs/cli/param/01_name.md) | Sole parameter definition (owned by `shader_chunks_query`) |
| [`table_plain` format](../../../../shader_chunks_query/docs/cli/format/01_table_plain.md) | Output format definition (owned by `shader_chunks_query`) |

### Tests

| File | Relationship |
|------|--------------|
| [../../../tests/docs/cli/command/readme.md](../../../tests/docs/cli/command/readme.md) | Command-level test specifications |

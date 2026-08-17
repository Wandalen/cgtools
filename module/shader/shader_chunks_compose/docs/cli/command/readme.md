# Commands

### Scope

- **Purpose:** Documents every command this crate exposes.
- **Responsibility:** One dedicated file per command with full syntax,
  parameters, examples, and cross-references.
- **In Scope:** The 1 command dispatched by this crate's contribution to
  `shader_chunks_cli_core`'s `CommandRegistry` — `compose`.
- **Out of Scope:** Parameter type/constraint detail (owned by
  `shader_chunks_query` — → [`names`](../../../../shader_chunks_query/docs/cli/param/02_names.md),
  [`transitive`](../../../../shader_chunks_query/docs/cli/param/09_transitive.md)),
  the other 7 commands of the `shader_chunks` family (→
  [family index](../../../../shader_chunks/docs/cli/readme.md)).

---

### Overview Table

| # | File | Command | Params | Status |
|---|------|---------|--------|--------|
| 1 | [01_compose.md](01_compose.md) | `.compose` | 1 | ✅ |

**Total:** 1 command (of 8 across the `shader_chunks` family)

### Commands Table

| # | Command | Purpose | Params | Group |
|---|---------|---------|--------|-------|
| 1 | `.compose` | Preview composed WGSL for the given chunks | 1 | [Compose](../command_group/01_compose.md) |

This crate's sole command belongs to the single-member `Compose` group
(see [`../command_group/`](../command_group/readme.md)). The remaining 7
commands of the `shader_chunks` family (`.list`, `.get`, `.tags`,
`.tree`, `.tunables`, `.preview`, `.render`) live in their own crates —
see the [family index](../../../../shader_chunks/docs/cli/readme.md).

### Docs

| File | Relationship |
|------|--------------|
| [../readme.md](../readme.md) | CLI documentation root (this crate) |
| [../format/readme.md](../format/readme.md) | Output format definitions |
| [../command_group/readme.md](../command_group/readme.md) | Command groups partitioning this layer |
| [`shader_chunks_query/docs/cli/param/readme.md`](../../../../shader_chunks_query/docs/cli/param/readme.md) | `names`/`transitive` parameter definitions (owned by `shader_chunks_query`) |

### Tests

| File | Relationship |
|------|--------------|
| [../../../tests/docs/cli/command/readme.md](../../../tests/docs/cli/command/readme.md) | Command-level test specifications |

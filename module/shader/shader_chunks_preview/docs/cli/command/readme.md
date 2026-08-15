# Commands

### Scope

- **Purpose:** Documents every command this crate exposes.
- **Responsibility:** One dedicated file per command with full syntax,
  parameters, examples, and cross-references.
- **In Scope:** The 1 command dispatched by this crate's contribution to
  `shader_chunks_cli_core`'s `CommandRegistry` — `preview`.
- **Out of Scope:** Parameter type/constraint detail (→ [`../param/`](../param/readme.md)),
  the other 7 commands of the `shader_chunks` family (→
  [family index](../../../../shader_chunks/docs/cli/readme.md)).

---

### Overview Table

| # | File | Command | Params | Status |
|---|------|---------|--------|--------|
| 1 | [01_preview.md](01_preview.md) | `.preview` | 3 | ✅ |

**Total:** 1 command (of 8 across the `shader_chunks` family)

### Commands Table

| # | Command | Purpose | Params | Group |
|---|---------|---------|--------|-------|
| 1 | `.preview` | Build, naga-validate, and optionally serve a live browser preview bundle | 3 (`name`, `file`, `serve`) | [Preview](../command_group/01_preview.md) |

This single command is its own [`command_group/`](../command_group/readme.md)
member — see that file's Semantic Coherence Test and "Why NOT Merge Into
Compose" rationale. Its 3 parameters split across crates: `name` is
owned by [`shader_chunks_query`](../../../../shader_chunks_query/docs/cli/param/01_name.md);
`file` and `serve` are this crate's own (see [`../param/`](../param/readme.md)).
The remaining 7 commands of the `shader_chunks` family live in their own
crates — see the [family index](../../../../shader_chunks/docs/cli/readme.md).

### Docs

| File | Relationship |
|------|--------------|
| [../readme.md](../readme.md) | CLI documentation root (this crate) |
| [../param/readme.md](../param/readme.md) | Parameter definitions |
| [../command_group/readme.md](../command_group/readme.md) | Command groups partitioning this layer |

### Tests

| File | Relationship |
|------|--------------|
| [../../../tests/docs/cli/command/readme.md](../../../tests/docs/cli/command/readme.md) | Command-level test specifications |

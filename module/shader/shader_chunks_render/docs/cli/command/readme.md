# Commands

### Scope

- **Purpose:** Documents every command this crate exposes.
- **Responsibility:** One dedicated file per command with full syntax,
  parameters, examples, and cross-references.
- **In Scope:** The 1 command dispatched by this crate's contribution to
  `shader_chunks_cli_core`'s `CommandRegistry` — `render`.
- **Out of Scope:** Parameter type/constraint detail (→ [`../param/`](../param/readme.md)),
  the other 7 commands of the `shader_chunks` family (→
  [family index](../../../../shader_chunks/docs/cli/readme.md)).

---

### Overview Table

| # | File | Command | Params | Status |
|---|------|---------|--------|--------|
| 1 | [01_render.md](01_render.md) | `.render` | 7 | ✅ |

**Total:** 1 command (of 9 across the `shader_chunks` family)

### Commands Table

| # | Command | Purpose | Params | Group |
|---|---------|---------|--------|-------|
| 1 | `.render` | Render one headless-GPU frame of a chunk's preview bundle to a static PNG (or every bundled chunk at once via `all::1`) | 7 | [Render](../command_group/01_render.md) |

`.render` accepts exactly one target — `name` (owned by
[`shader_chunks_query`](../../../../shader_chunks_query/docs/cli/param/01_name.md))
or `file::` (owned by
[`shader_chunks_preview`](../../../../shader_chunks_preview/docs/cli/param/01_file.md))
— plus this crate's own [`out`](../param/01_out.md), [`size`](../param/02_size.md),
[`time`](../param/03_time.md), [`set`](../param/04_set.md), and
[`all`](../param/05_all.md) (mutually exclusive with `name`, `file::`,
and `set::`). The remaining 7 commands of the `shader_chunks` family
live in their own crates — see the
[family index](../../../../shader_chunks/docs/cli/readme.md).

### Docs

| File | Relationship |
|------|--------------|
| [../readme.md](../readme.md) | CLI documentation root (this crate) |
| [../param/readme.md](../param/readme.md) | Parameter definitions owned by this crate |
| [../type/readme.md](../type/readme.md) | Type definitions owned by this crate |
| [../command_group/readme.md](../command_group/readme.md) | Command group partitioning this layer |

### Tests

| File | Relationship |
|------|--------------|
| [../../../tests/docs/cli/command/readme.md](../../../tests/docs/cli/command/readme.md) | Command-level test specifications |

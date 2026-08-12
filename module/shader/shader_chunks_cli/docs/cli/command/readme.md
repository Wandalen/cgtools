# Commands

### Scope

- **Purpose:** Documents every command `shader_chunks_cli` exposes.
- **Responsibility:** One dedicated file per command with full syntax,
  parameters, examples, and cross-references.
- **In Scope:** The 5 commands dispatched by `src/main.rs`'s
  `CommandRegistry`.
- **Out of Scope:** Parameter type/constraint detail (→ [`../param/`](../param/readme.md)),
  output rendering detail (→ [`../format/`](../format/readme.md)).

---

### Overview Table

| # | File | Command | Params | Status |
|---|------|---------|--------|--------|
| 1 | [01_list.md](01_list.md) | `.list` | 0 | ✅ |
| 2 | [02_get.md](02_get.md) | `.get` | 1 | ✅ |
| 3 | [03_tags.md](03_tags.md) | `.tags` | 0 | ✅ |
| 4 | [04_tree.md](04_tree.md) | `.tree` | 1 | ✅ |
| 5 | [05_compose.md](05_compose.md) | `.compose` | 1 | ✅ |

**Total:** 5 commands

### Commands Table

| # | Command | Purpose | Params | Namespace |
|---|---------|---------|--------|-----------|
| 1 | `.list` | List every bundled shader chunk | 0 | [chunk](../command_group.md) |
| 2 | `.get` | Show full detail for one chunk | 1 | [chunk](../command_group.md) |
| 3 | `.tags` | List every distinct tag and its chunk(s) | 0 | [chunk](../command_group.md) |
| 4 | `.tree` | Show a chunk's dependency tree, or the full forest | 1 | [chunk](../command_group.md) |
| 5 | `.compose` | Preview composed WGSL for the given chunks | 1 | [chunk](../command_group.md) |

All 5 commands belong to the single [`Inspection`](../command_group.md)
command group — see that file for the group's Semantic Coherence Test.

### Docs

| File | Relationship |
|------|--------------|
| [../readme.md](../readme.md) | CLI documentation root |
| [../param/readme.md](../param/readme.md) | Parameter definitions |
| [../type/readme.md](../type/readme.md) | Type definitions |
| [../format/readme.md](../format/readme.md) | Output format definitions |
| [../command_group.md](../command_group.md) | Command group this layer belongs to |

### Tests

| File | Relationship |
|------|--------------|
| [../../../tests/docs/cli/command/readme.md](../../../tests/docs/cli/command/readme.md) | Command-level test specifications |

# Commands

### Scope

- **Purpose:** Documents every command `shader_chunks` exposes.
- **Responsibility:** One dedicated file per command with full syntax,
  parameters, examples, and cross-references.
- **In Scope:** The 6 commands dispatched by `src/cli.rs`'s
  `CommandRegistry`.
- **Out of Scope:** Parameter type/constraint detail (→ [`../param/`](../param/readme.md)),
  output rendering detail (→ [`../format/`](../format/readme.md)).

---

### Overview Table

| # | File | Command | Params | Status |
|---|------|---------|--------|--------|
| 1 | [01_list.md](01_list.md) | `.list` | 20 | ✅ |
| 2 | [02_get.md](02_get.md) | `.get` | 20 | ✅ |
| 3 | [03_tags.md](03_tags.md) | `.tags` | 0 | ✅ |
| 4 | [04_tree.md](04_tree.md) | `.tree` | 1 | ✅ |
| 5 | [05_compose.md](05_compose.md) | `.compose` | 1 | ✅ |
| 6 | [06_tunables.md](06_tunables.md) | `.tunables` | 1 | ✅ |

**Total:** 6 commands

### Commands Table

| # | Command | Purpose | Params | Group |
|---|---------|---------|--------|-------|
| 1 | `.list` | Query bundled chunks — filter, sort, project, format; every chunk by default | 20 | [Query](../command_group/01_query.md) |
| 2 | `.get` | Same query engine, names required, detail defaults | 20 | [Query](../command_group/01_query.md) |
| 3 | `.tags` | List every distinct tag and its chunk(s) | 0 | [Query](../command_group/01_query.md) |
| 4 | `.tree` | Show a chunk's dependency tree, or the full forest | 1 | [Graph](../command_group/02_graph.md) |
| 5 | `.compose` | Preview composed WGSL for the given chunks | 1 | [Compose](../command_group/03_compose.md) |
| 6 | `.tunables` | List a chunk's declared tunable parameters | 1 | [Parameters](../command_group/04_parameters.md) |

The 6 commands partition into the 4
[`command_group/`](../command_group/readme.md) groups above — each group
file carries its own Semantic Coherence Test. `.list` and `.get` share
one routine and one parameter surface; their 20 declared parameters are
the same 20 (see [`../param_group/`](../param_group/readme.md)).

### Docs

| File | Relationship |
|------|--------------|
| [../readme.md](../readme.md) | CLI documentation root |
| [../param/readme.md](../param/readme.md) | Parameter definitions |
| [../param_group/readme.md](../param_group/readme.md) | Shared parameter set of the query commands |
| [../type/readme.md](../type/readme.md) | Type definitions |
| [../format/readme.md](../format/readme.md) | Output format definitions |
| [../command_group/readme.md](../command_group/readme.md) | Command groups partitioning this layer |

### Tests

| File | Relationship |
|------|--------------|
| [../../../tests/docs/cli/command/readme.md](../../../tests/docs/cli/command/readme.md) | Command-level test specifications |

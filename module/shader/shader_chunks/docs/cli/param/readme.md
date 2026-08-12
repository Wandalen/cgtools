# Parameters

### Scope

- **Purpose:** Documents every parameter accepted by any `shader_chunks` command.
- **Responsibility:** One dedicated file per parameter, unified across every command that accepts it.
- **In Scope:** The 2 parameters declared across the 5 commands' `ArgumentDefinition`s.
- **Out of Scope:** Command-level syntax/examples (→ [`../command/`](../command/readme.md)), type constraints/parsing (→ [`../type/`](../type/readme.md)).

---

### Overview Table

| # | File | Parameter | Type | Default | Status |
|---|------|-----------|------|---------|--------|
| 1 | [01_name.md](01_name.md) | `name` | [`ChunkName`](../type/01_chunk_name.md) | **(required)** for `get`; `Varies` for `tree` | ✅ |
| 2 | [02_names.md](02_names.md) | `names` | [`ChunkName`](../type/01_chunk_name.md) (list) | **(required)** | ✅ |

**Total:** 2 parameters

**Parameter Groups:** none — no two commands share a *set* of ≥2
co-occurring parameters (`param_group/` deliberately omitted; see
[`../readme.md` § Scope Decisions](../readme.md#scope-decisions)).

### Docs

| File | Relationship |
|------|--------------|
| [../readme.md](../readme.md) | CLI documentation root |
| [../command/readme.md](../command/readme.md) | Commands accepting these parameters |
| [../type/readme.md](../type/readme.md) | Type definitions |

### Tests

| File | Relationship |
|------|--------------|
| [../../../tests/docs/cli/param/readme.md](../../../tests/docs/cli/param/readme.md) | Parameter-level test specifications |

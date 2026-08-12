# Types

### Scope

- **Purpose:** Documents every semantic parameter type `shader_chunks` uses.
- **Responsibility:** One dedicated file per type — purpose, fundamental representation, constraints, parsing, methods.
- **In Scope:** The 1 domain type this CLI operates on.
- **Out of Scope:** Per-parameter defaults/requiredness (→ [`../param/`](../param/readme.md)).

---

### Overview Table

| # | File | Type | Fundamental | Status |
|---|------|------|-------------|--------|
| 1 | [01_chunk_name.md](01_chunk_name.md) | ChunkName | `String` | ✅ |

**Total:** 1 type

### Docs

| File | Relationship |
|------|--------------|
| [../readme.md](../readme.md) | CLI documentation root |
| [../param/readme.md](../param/readme.md) | Parameters carrying this type |
| [../command/readme.md](../command/readme.md) | Commands using this type via a parameter |

### Tests

| File | Relationship |
|------|--------------|
| [../../../tests/docs/cli/type/readme.md](../../../tests/docs/cli/type/readme.md) | Type-level test specifications |

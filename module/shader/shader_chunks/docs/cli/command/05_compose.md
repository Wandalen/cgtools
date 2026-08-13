# Command :: 5. compose

### Description

Previews the WGSL text produced by composing one or more chunks,
dependency-ordered regardless of input order. Use it to check the final
composed output before wiring `shader_chunks_core::try_compose` into a real
pipeline.

-- **Parameters:** names, transitive
-- **Exit Codes:** 0 (success) | 1 (a name does not resolve, or a resolved
   set is missing a declared dependency, or contains a cyclic dependency)
-- **Modes:** (none)

### Syntax
```bash
shader_chunks compose <names...> [param::value ...]
```

### Parameters

| Parameter | Type | Default | Required | Purpose |
|-----------|------|---------|----------|---------|
| `names` | [`ChunkName`](../param/02_names.md) (list) | — | Yes | One or more chunks to compose, in any order |
| `transitive` | [`Switch`](../param/09_transitive.md) | `false` | No | Widen the named set to its full dependency closure |

### Examples
```bash
shader_chunks compose hash21 value_noise
# fn hash21(p: vec2f) -> f32 { /* ... */ }
#
# fn value_noise(p: vec2f) -> f32 { /* ... uses hash21 ... */ }

shader_chunks compose value_noise
# (value_noise depends on hash21, which was not supplied)
# value_noise depends on hash21, which was not included
# (exit 1)

shader_chunks compose fbm3 transitive::1
# (pulls value_noise and hash21 unasked; identical output to
#  `compose hash21 value_noise fbm3`)
```

### Notes
- Order-independent: `compose hash21 value_noise` and
  `compose value_noise hash21` produce identical output — dependency order
  is resolved internally by `shader_chunks_core::try_compose`, never by argument
  order.
- Strict by default: the named set must already be dependency-complete —
  a missing dependency is a loud exit-1 error, never silently pulled in.
  [`transitive::`](../param/09_transitive.md) opts into the closure walk
  explicitly.
- Uses the fallible `shader_chunks_core::try_compose` (task 099's API), never
  the panicking `shader_chunks_core::compose` — a missing dependency or cyclic
  input reports `CliError::Compose` and exits 1, it never panics.
- Output format: [`plain_text`](../format/03_plain_text.md).

### Related Commands

| # | Command | Relationship |
|---|---------|--------------|
| 1 | [`.tree`](04_tree.md) | Preview the dependency order before composing |
| 2 | [`.get`](02_get.md) | Inspect one chunk's exports before including it here |

### Referenced User Stories

*(None — this project deliberately omits the `user_story/` collection at
this CLI's scale; see [`docs/cli/readme.md` § Scope
Decisions](../readme.md#scope-decisions).)*

---

**Category:** chunk
**Complexity:** 3
**API Requirement:** None
**Idempotent:** Yes
**Risk Level:** Low

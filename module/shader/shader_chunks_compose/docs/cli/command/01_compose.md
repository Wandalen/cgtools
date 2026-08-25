# Command :: 5. compose

### Description

Previews the WGSL text produced by composing one or more chunks,
dependency-ordered regardless of input order. Use it to check the final
composed output before wiring `shader_chunks_core::try_compose` into a real
pipeline. With `out::<path>`, writes the composed WGSL to a file instead
of printing it.

-- **Parameters:** names, transitive, out
-- **Exit Codes:** 0 (success) | 1 (a name does not resolve, or a resolved
   set is missing a declared dependency, or contains a cyclic dependency)
   | 2 (`out::<path>` could not be written, e.g. a missing parent
   directory)
-- **Modes:** (none)

### Syntax
```bash
shader_chunks compose <names...> [param::value ...]
```

### Parameters

| Parameter | Type | Default | Required | Purpose |
|-----------|------|---------|----------|---------|
| `names` | [`ChunkName`](../../../../shader_chunks_query/docs/cli/param/02_names.md) (list) | — | Yes | One or more chunks to compose, in any order |
| `transitive` | [`Switch`](../../../../shader_chunks_query/docs/cli/param/09_transitive.md) | `false` | No | Widen the named set to its full dependency closure |
| `out` | [`String`](../param/01_out.md) | None (prints to stdout) | No | Write the composed WGSL to this file instead of printing it |

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

shader_chunks compose fbm3 transitive::1 out::fbm3_bundle.wgsl
# wrote fbm3_bundle.wgsl (712 bytes wgsl)
# (composed text goes to the file, not stdout; byte count varies)
```

### Notes
- Order-independent: `compose hash21 value_noise` and
  `compose value_noise hash21` produce identical output — dependency order
  is resolved internally by `shader_chunks_core::try_compose`, never by argument
  order.
- Strict by default: the named set must already be dependency-complete —
  a missing dependency is a loud exit-1 error, never silently pulled in.
  [`transitive::`](../../../../shader_chunks_query/docs/cli/param/09_transitive.md) opts into the closure walk
  explicitly.
- Uses the fallible `shader_chunks_core::try_compose` (task 099's API), never
  the panicking `shader_chunks_core::compose` — a missing dependency or cyclic
  input reports `CliError::Compose` and exits 1, it never panics.
- [`out::`](../param/01_out.md) writes the composed text to a file instead
  of stdout; it is only ever attempted after composition already
  succeeded, so a name/dependency/cycle failure never reaches the write
  step and never leaves a partial file. A write failure (e.g. a missing
  parent directory) is a separate `ComposeCliError::Io`, exit 2 — the
  only way this command exits 2 rather than 1.
- Output format: [`plain_text`](../format/01_plain_text.md).

### Related Commands

| # | Command | Relationship |
|---|---------|--------------|
| 1 | [`.tree`](../../../../shader_chunks_query/docs/cli/command/04_tree.md) | Preview the dependency order before composing |
| 2 | [`.get`](../../../../shader_chunks_query/docs/cli/command/02_get.md) | Inspect one chunk's exports before including it here |

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

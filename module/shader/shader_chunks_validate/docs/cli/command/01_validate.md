# Command :: 9. validate

### Description

Runs five independent, non-panicking registry-wide integrity checks over
every chunk in [`shader_chunks_core::CHUNKS`](../../../../shader_chunks_core/readme.md)
in one pass, reporting every finding rather than stopping at the first
one: manifest drift, duplicate names, missing dependencies, dependency
cycles, and naga WGSL compilation. A clean registry prints an explicit
all-clear message; a dirty one prints one block per finding and exits
non-zero.

-- **Parameters:** (none)
-- **Exit Codes:** 0 (success — zero findings) | 1 (one or more findings)
-- **Modes:** (none)

### Syntax
```bash
shader_chunks validate
```

### Parameters

*(none)*

### Examples
```bash
shader_chunks validate
# registry is clean: 50 chunks, 0 findings
```

The real bundled registry is clean today, so the only outcome reachable
via subprocess against it is the all-clear message above. When one or
more checks report a finding, `validate` instead prints a count header
followed by one `[chunk] check: message` block per finding, blank-line
separated:

```text
2 finding(s):

[hash21] manifest_drift: description field disagrees with the manifest

[fbm3] wgsl_compile: <naga diagnostic text, possibly multi-line>
```

See `shader_chunks_validate_core/tests/shader_chunks_validate_core_test.rs`
for a fixture-driven worked example of each individual check, and
`shader_chunks_validate/tests/validate_cli_test.rs` for the rendered
report shapes themselves.

### Notes
- The five checks, each producing a distinct `check` label: `manifest_drift`
  (a chunk's compiled-in descriptor fields disagree with what
  `shader_chunks_core::manifest_mismatches` freshly parses from its own
  WGSL text), `duplicate_name` (two chunks share a `//@ name:`, which
  would silently shadow one behind `shader_chunks_core::chunk_get`'s
  first-match lookup), `missing_dependency` (a `//@ depends_on:` entry
  names a chunk absent from the registry), `dependency_cycle` (the
  registry cannot be topologically sorted), and `wgsl_compile` (a
  chunk's own transitive dependency closure, composed, fails naga parse
  or validation — the same front end `wgpu` uses).
- Checks never double-report the same root problem under two labels: a
  missing dependency is reported once as `missing_dependency`, never
  again as a derivative `dependency_cycle`; a genuine cycle is reported
  once as `dependency_cycle`, never again as a derivative
  `wgsl_compile` failure.
- A dependency-only chunk with no `//@ stage:` entry point (most of the
  bundled registry — plain building blocks like `hash21`) still passes
  `wgsl_compile` cleanly: a WGSL module containing only free functions
  and no `@vertex`/`@fragment`/`@compute` stage is itself valid WGSL.
- **Deliberate scope cut:** `//@ param:` line malformation is not
  checked. Discovering that requires `shader_chunks_params_core::discover`,
  which panics by design on a malformed line rather than returning a
  `Result` (chunk manifests are trusted authored content, not
  adversarial input — see `shader_chunks_validate_core`'s own module
  doc comment for the full rationale).
- Output format: [`plain_text`](../../../../shader_chunks_compose/docs/cli/format/01_plain_text.md)
  — unstructured text, never a table, since a `wgsl_compile` finding's
  naga diagnostic is typically multi-line and would corrupt a table
  cell.

### Related Commands

| # | Command | Relationship |
|---|---------|--------------|
| 1 | [`.compose`](../../../../shader_chunks_compose/docs/cli/command/01_compose.md) | Composes one selected chunk set's WGSL; `validate` naga-checks every chunk's own closure across the whole registry instead |
| 2 | [`.render`](../../../../shader_chunks_render/docs/cli/command/01_render.md) | Also naga-validates as a side effect of producing an artifact; `validate` produces no artifact, only a diagnostic report |

### Referenced User Stories

*(None — this project deliberately omits the `user_story/` collection at
this CLI's scale; see [`docs/cli/readme.md` § Scope
Decisions](../readme.md#scope-decisions).)*

---

**Category:** chunk
**Complexity:** 2
**API Requirement:** None
**Idempotent:** Yes
**Risk Level:** Low

# shader_chunks_validate_core

**Keywords:** WGSL, Shader Composition, Registry Linting, Integrity Checks

Registry-wide integrity checks over
[`shader_chunks_core::CHUNKS`](../shader_chunks_core/readme.md) — the
engine behind `shader_chunks validate`. Five independent, non-panicking
checks run across every bundled chunk in one pass and report every
problem found, rather than failing loudly (`compose`'s panic) or
stopping at the first one:

- **Manifest drift** — a chunk's compiled-in descriptor fields disagree
  with what `shader_chunks_core::manifest_mismatches` freshly parses from
  the chunk's own `wgsl` text.
- **Duplicate names** — two bundled chunks share a `//@ name:`, which
  would silently shadow one of them behind
  `shader_chunks_core::chunk_get`'s first-match lookup.
- **Missing dependencies** — a `//@ depends_on:` entry names a chunk not
  present anywhere in the bundled registry.
- **Dependency cycles** — the registry cannot be topologically sorted.
- **WGSL compilation** — a chunk's own transitive dependency closure,
  composed, fails naga parse or validation (the same front end `wgpu`
  uses, reused from `shader_chunks_preview`'s `bundle_prepare` check but
  scoped to raw composed text rather than a full preview bundle, so a
  dependency-only chunk with no previewable export is still checked).

Deliberately out of scope: `//@ param:` line malformation. Discovering
that requires `shader_chunks_params_core::discover`, which panics rather
than returning a `Result` on a malformed line (by design — chunk
manifests are trusted authored content, not adversarial input, matching
`shader_chunks_core`'s own `manifest_field` panic-on-malformed idiom).

## Usage

```rust
use shader_chunks_validate_core::validate_registry;

let findings = validate_registry();
assert!( findings.is_empty(), "bundled registry is clean" );
```

`validate( chunks )` runs the same five checks over an arbitrary chunk
slice rather than the bundled registry alone, so tests can exercise each
check against self-contained fixtures without any bundled chunk needing
to be broken. `validate_registry()` is the bundled-registry convenience
wrapper `shader_chunks validate` actually calls.

Every `Finding` carries the offending chunk's name (or `"(registry)"`
for a whole-registry problem like a dependency cycle spanning several
chunks), which of the five checks found it, and a human-readable
message. Checks never double-report the same root problem under two
labels: a missing dependency is reported once as `missing_dependency`,
never again as a derivative `dependency_cycle`; a genuine cycle is
reported once as `dependency_cycle`, never again as a derivative
`wgsl_compile` failure.

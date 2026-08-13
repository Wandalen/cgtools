# Invariant: Descriptor-Manifest Parity

A `ChunkDescriptor` never lies about the WGSL it carries: every metadata
field equals what the manifest parsers read from that descriptor's own
`wgsl` text. The manifest inside the file and the compile-time data about
the file are one fact, not two.

### Scope

- **Purpose**: Pin that descriptor fields are a faithful mirror of the `//@` manifest, so nothing downstream ever needs to re-parse — or doubt — them.
- **Responsibility**: State the property, enumerate its enforcement for bundled and local rows, and record what breaks when it fails.
- **In Scope**: All six mirrored fields — `name`, `description`, `tags`, `stage`, `depends_on`, `exports` — for every descriptor, bundled or local.
- **Out of Scope**: Whether the manifest itself matches the WGSL body under it (held by the crate's `depends_on_covers_…`/`export_names_match_…` header-honesty tests); how bundled rows are produced (see [../algorithm/001_registry_generation.md](../algorithm/001_registry_generation.md)).

### Invariant Statement

For every `ChunkDescriptor` `c`, and for each of the six metadata fields,
the field's value equals what the corresponding `parse_*` function
(`parse_name`, `parse_description`, `parse_tags`, `parse_stage`,
`parse_depends_on`, `parse_exports`) returns for `c.wgsl`. This always
holds — for every bundled row, and for every local row in a crate that
carries the pattern's guard test.

### Enforcement Mechanism

- **Bundled rows, by construction**: `build.rs` generates each row directly
  from the manifest it mirrors and asserts the `//@ name:` value equals the
  chunk's directory name — the row cannot start out drifted.
- **Bundled rows, by test**: `chunks_table_matches_each_manifest` re-derives
  every field of every `CHUNKS` row through the lib's own `parse_*` readers
  and asserts equality — which simultaneously guards the build script's
  deliberately duplicated minimal parser against drifting from the lib
  parsers.
- **Local rows**: `manifest_mismatches( &chunk )` compares all six fields
  the same way and returns one message per drifted field; a crate defining
  local chunks holds the invariant with one test per chunk asserting the
  result is empty. `manifest_mismatches_reports_every_drifted_field` proves
  the guard itself reports all six fields, not a subset.

### Violation Consequences

- Composition order is computed from `depends_on` fields — a drifted
  `depends_on` reorders or breaks composition while the manifest looks
  correct in review.
- `dependency_closed` would validate the wrong dependency graph, letting an
  incomplete set pass its compile-time check.
- Tooling and humans reading descriptors (the `sch` CLI's queries, an
  `exports` signature copied into calling code) would be misled by metadata
  that no longer describes the shader text shipped beside it.

### Example

Change a local descriptor's `depends_on` to `&[ "hash21" ]` while its
manifest still declares `//@ depends_on: value_noise` and the guard reports
exactly the drifted field:

```text
chunk `my_glow`: descriptor depends_on ["hash21"] != manifest `//@ depends_on:` value ["value_noise"]
```

Verify each half directly: run
`cargo nextest run -p shader_chunks_core chunks_table_matches_each_manifest`
for the bundled table, and
`cargo nextest run -p shader_chunks_core manifest_mismatches_reports` to see
the local-row guard exercise all six fields.

### Algorithms

| File | Relationship |
|------|--------------|
| [../algorithm/001_registry_generation.md](../algorithm/001_registry_generation.md) | Generation-from-manifest is the by-construction half of enforcement for bundled rows |

### Patterns

| File | Relationship |
|------|--------------|
| [../pattern/002_crate_local_chunk.md](../pattern/002_crate_local_chunk.md) | The consumer form whose hand-written rows this invariant's `manifest_mismatches` guard keeps honest |

### Sources

| File | Relationship |
|------|--------------|
| `src/lib.rs` | `manifest_mismatches` and the `parse_*` readers that define the manifest side of the equality |
| `build.rs` | Emits bundled rows straight from manifests; asserts manifest name equals directory name |

### Tests

| File | Relationship |
|------|--------------|
| `tests/shader_chunks_core_test.rs` | `chunks_table_matches_each_manifest` (bundled), `manifest_mismatches_reports_every_drifted_field` (guard completeness), `local_chunk_descriptor_matches_its_manifest` (local fixture) |
| `examples/orrery/webgpu/tests/shader_source_test.rs` (repo root) | `scene_fragment_descriptor_matches_its_manifest` — the invariant held in a real consumer |

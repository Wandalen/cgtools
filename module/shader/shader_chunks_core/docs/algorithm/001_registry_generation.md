# Algorithm: Registry Generation

The bundled-chunk table `CHUNKS` and its O(1) lookup `chunk_get` are not
handwritten — `build.rs` regenerates them on every build from the repo-root
`shader/` collection, so adding a chunk to the collection needs no Rust edit
in this crate.

### Scope

- **Purpose**: Make the Rust registry a projection of the `shader/` collection rather than a second, hand-maintained copy of it.
- **Responsibility**: Describe the generation procedure — its two input sources, their cross-validation, and the emitted artifacts.
- **In Scope**: What `build.rs` reads, the order it imposes, every way it can fail the build, and what `$OUT_DIR/chunks.rs` contains.
- **Out of Scope**: The field-level equality between generated rows and manifests (see [../invariant/002_descriptor_manifest_parity.md](../invariant/002_descriptor_manifest_parity.md)); how applications consume the table (see [../pattern/001_selective_const_import.md](../pattern/001_selective_const_import.md)).

### Abstract

The registry has two independent input sources, deliberately: **membership**
comes from scanning the collection's chunk directories
(`shader/<name>/<name>.wgsl`), while **row order** comes from the
human-curated collection-index table in `shader/readme.md`. Order is
load-bearing — registry consumers observe it (the `shader_chunks` CLI's
default `sort::input` is documented as registry order) — so it belongs to a
human-editable document, not to filesystem enumeration order. The two
sources are cross-validated both ways; any disagreement fails the build
naming the offender, so the index can never silently drift from the
collection it indexes.

The generation relies on one external guarantee: `build.rs` registers the
whole `shader/` tree with `cargo::rerun-if-changed`, so cargo re-runs it on
any manifest, body, or index edit.

### Algorithm

1. Resolve the collection directory `shader/` relative to
   `CARGO_MANIFEST_DIR` and register it (recursively) for rerun-on-change.
2. **Scan membership**: every subdirectory of `shader/` is a chunk and must
   contain `<name>.wgsl` — a subdirectory without one fails the build.
3. **Read order**: parse the collection-index table in `shader/readme.md`;
   each row's first cell (`[<name>/](…)`) names one chunk, in table order.
4. **Cross-validate**: a duplicate index row, a scanned directory missing
   from the index, or an index row without a directory each fail the build
   with a message naming the exact chunk.
5. For each name in index order: read its WGSL, parse the `//@` manifest
   header (the `name:` value must equal the directory name), and emit one
   `ChunkDescriptor` row — every manifest field as Rust literals, the source
   text as `include_str!` anchored to `CARGO_MANIFEST_DIR` — plus one
   `chunk_get` match arm mapping the name to its row index.
6. Write the result to `$OUT_DIR/chunks.rs`; `src/lib.rs` splices it into
   `mod private` with `include!`.

The `//@` parsing in step 5 deliberately re-implements the minimal subset of
the lib's `parse_*` functions, because a build script cannot call the crate
it is building; the parity invariant's drift-guard test holds the two
implementations equal.

### Example

Adding a fifth chunk `ridge` takes two collection edits and zero Rust edits:
create `shader/ridge/ridge.wgsl` opening with its `//@` manifest, and add
its row to the index table in `shader/readme.md`. The next `cargo build`
regenerates `CHUNKS` with five rows and a `"ridge"` match arm in
`chunk_get`. Forget the index row and the build fails with:

```text
chunk directory `ridge` is missing from the collection-index table in .../shader/readme.md
```

Add the row but not the directory and it fails with the opposite message
(`collection-index row `ridge` … has no `shader/ridge/` directory`). Both
directions are cheap to verify live: add a bogus row to the index table, run
`cargo check -p shader_chunks_core`, watch it fail naming the row, then
revert.

### Invariants

| File | Relationship |
|------|--------------|
| [../invariant/002_descriptor_manifest_parity.md](../invariant/002_descriptor_manifest_parity.md) | Generation-from-manifest is the bundled half of this parity guarantee; its test also guards this algorithm's duplicated parser |

### Patterns

| File | Relationship |
|------|--------------|
| [../pattern/001_selective_const_import.md](../pattern/001_selective_const_import.md) | The generated table is the set `chunk( name )` imports from |

### Sources

| File | Relationship |
|------|--------------|
| `build.rs` | The generator: scan, index parse, cross-validation, row emission |
| `src/lib.rs` | Splices `$OUT_DIR/chunks.rs` into `mod private` via `include!` |
| `shader/readme.md` (repo root) | The collection-index table supplying row order |

### Tests

| File | Relationship |
|------|--------------|
| `tests/shader_chunks_core_test.rs` | `chunks_table_lists_every_bundled_chunk`, `chunks_table_matches_each_manifest`, and `chunk_get_resolves_every_bundled_name_to_its_row` pin the generated artifacts |

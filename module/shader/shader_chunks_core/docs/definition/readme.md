# Doc Definitions

Crate-scope design documentation for `shader_chunks_core` — the compile-time
chunk-import machinery, its composition algorithm, and the guarantees they
rest on. Anything spanning multiple crates lives in the repo-root `docs/`
instead.

## Master Doc Definitions Table

| Type | Purpose | Master File | Instances |
|------|---------|-------------|----------:|
| `algorithm/` | Build-time registry generation and dependency-ordered composition procedures | [algorithm/readme.md](../algorithm/readme.md) | 2 |
| `invariant/` | Correctness properties that must always hold, and their enforcement mechanisms | [invariant/readme.md](../invariant/readme.md) | 2 |
| `pattern/` | Reusable consumer-side forms for selecting bundled chunks and defining local ones | [pattern/readme.md](../pattern/readme.md) | 2 |

## Master Doc Instances Table

| Definition | ID | Name | File |
|-----------|-----|------|------|
| algorithm | 001 | Registry Generation | [algorithm/001_registry_generation.md](../algorithm/001_registry_generation.md) |
| algorithm | 002 | Dependency-Ordered Composition | [algorithm/002_dependency_ordered_composition.md](../algorithm/002_dependency_ordered_composition.md) |
| invariant | 001 | Dependency Closure | [invariant/001_dependency_closure.md](../invariant/001_dependency_closure.md) |
| invariant | 002 | Descriptor-Manifest Parity | [invariant/002_descriptor_manifest_parity.md](../invariant/002_descriptor_manifest_parity.md) |
| pattern | 001 | Selective Const Import | [pattern/001_selective_const_import.md](../pattern/001_selective_const_import.md) |
| pattern | 002 | Crate-Local Chunk | [pattern/002_crate_local_chunk.md](../pattern/002_crate_local_chunk.md) |

# Pattern: Crate-Local Chunk

A consumer defines its own, non-reusable chunk — typically the scene's
fragment stage — as a plain `ChunkDescriptor` literal over `include_str!`,
the same shape as the bundled rows. Local and imported rows then mix freely
in one set and compose as one shader.

### Scope

- **Purpose**: Let app-specific WGSL participate in chunk composition without entering the shared collection it doesn't belong in.
- **Responsibility**: Describe the local-definition form, why mixing is seamless, its applicability bound, and its drift-guard obligation.
- **In Scope**: Descriptor literals in consumer crates, their manifest mirror, and mixed-set membership.
- **Out of Scope**: How the mixed set is ordered and concatenated (see [../algorithm/002_dependency_ordered_composition.md](../algorithm/002_dependency_ordered_composition.md)); the field-parity guarantee the guard test provides (see [../invariant/002_descriptor_manifest_parity.md](../invariant/002_descriptor_manifest_parity.md)).

### Problem

A scene's fragment stage composes with the shared noise chunks but is not
reusable — publishing it to the repo-root `shader/` collection would pollute
a curated, reusable set with app-specific code. Yet if local WGSL were a
different kind of thing than bundled chunks, composition would need two code
paths, and the local half would lose the ordering and validation guarantees
the bundled half enjoys.

### Solution

There is only one kind of thing. `ChunkDescriptor` is public with all-public
fields, so a consumer constructs one in a `const`, mirroring the `//@`
manifest that still opens its own WGSL file, with `wgsl :
include_str!( … )` bundling the source. Provenance is invisible downstream:
a set is `&[ ChunkDescriptor ]` regardless of where each row came from, and
every consumer of descriptors — composition, `chunk_get_from`,
`dependency_closed` — reads only fields. A local chunk may depend on bundled
chunks (and vice versa would work identically) because dependencies are
names resolved within the set, not links to the registry.

The mirror comes with an obligation: nothing generates a local descriptor
from its manifest, so each local chunk carries one test asserting
`manifest_mismatches( &CHUNK )` is empty — the same parity the bundled
table gets from its build-time generation.

### Applicability

Use for single-consumer WGSL: scene fragments, app-specific effects,
anything with exactly one caller. The moment a local chunk earns a second
consumer, graduate it to the `shader/` collection instead — the registry
generation makes that a two-edit move with no Rust changes here.

### Consequences

- The shared collection stays purely reusable; app code stays in the app.
- Local chunks get identical treatment: dependency ordering, compile-time
  set validation, `const` selection — no second-class path.
- The manifest mirror is maintained by hand and honest only through its
  parity test; skipping the test silently re-opens the drift the bundled
  table is guarded against.
- The local WGSL file must carry a real `//@` manifest even though no
  generator reads it — the cost of keeping one convention across both row
  sources.

### Example

The orrery scene's fragment stage, abridged from its live source:

```rust
use shader_chunks_core::{ chunk, ChunkDescriptor };

const SCENE_FRAGMENT : ChunkDescriptor = ChunkDescriptor
{
  name : "scene_fragment",
  description : "Sun-grid-lines HUD scene fragment stage: …",
  tags : &[ ( "category", "scene" ) ],
  stage : Some( "fragment" ),
  depends_on : &[ "hash21", "fbm3", "fullscreen_triangle" ],
  exports : &[ "fn fs_main(in: VertexOutput) -> @location(0) vec4f" ],
  wgsl : include_str!( "../shader/scene_fragment.wgsl" ),
};

const SCENE_CHUNKS : &[ ChunkDescriptor ] =
&[
  chunk( "hash21" ),
  chunk( "value_noise" ),
  chunk( "fbm3" ),
  chunk( "fullscreen_triangle" ),
  SCENE_FRAGMENT, // local and imported rows mix freely
];
```

Its parity test is one line of substance:
`assert!( manifest_mismatches( &SCENE_FRAGMENT ).is_empty() )`. Verify the
guard bites: edit one word of the `description` field (or the manifest line
it mirrors) and run
`cargo nextest run -p orrery_webgpu scene_fragment_descriptor` — the test
fails printing the exact drifted field; revert.

### Algorithms

| File | Relationship |
|------|--------------|
| [../algorithm/002_dependency_ordered_composition.md](../algorithm/002_dependency_ordered_composition.md) | Composes mixed sets seamlessly by reading only the descriptor fields this pattern fills in |

### Invariants

| File | Relationship |
|------|--------------|
| [../invariant/002_descriptor_manifest_parity.md](../invariant/002_descriptor_manifest_parity.md) | The per-chunk `manifest_mismatches` test extends this guarantee to hand-written local rows |

### Patterns

| File | Relationship |
|------|--------------|
| [001_selective_const_import.md](001_selective_const_import.md) | The complementary row source: bundled chunks imported by name into the same set |

### Sources

| File | Relationship |
|------|--------------|
| `src/lib.rs` | `ChunkDescriptor`'s public fields — the shape local literals fill in — and `manifest_mismatches` |
| `examples/orrery/webgpu/src/shader_source.rs` (repo root) | Known use: `SCENE_FRAGMENT` local chunk mixed into `SCENE_CHUNKS` |
| `examples/orrery/webgpu/shader/scene_fragment.wgsl` (repo root) | The local chunk's WGSL, opening with the manifest the descriptor mirrors |

### Tests

| File | Relationship |
|------|--------------|
| `tests/shader_chunks_core_test.rs` | Mixed-set fixtures (`local_chunk_descriptor_matches_its_manifest`, `compose_set_orders_a_mixed_set_dependency_before_dependent`) prove the pattern in isolation |
| `examples/orrery/webgpu/tests/shader_source_test.rs` (repo root) | `scene_fragment_descriptor_matches_its_manifest` is the live drift-guard this pattern obligates |

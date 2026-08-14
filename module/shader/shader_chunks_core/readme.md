# shader_chunks_core

**Keywords:** WGSL, Shader Composition, Shader Manifest, Procedural Generation, Noise

Manifest-driven WGSL shader-chunk composition: small, reusable pieces of WGSL
stored one per file, each carrying a machine-parsable header describing its
interface, composed into a complete shader source at runtime by
`shader_chunks_core::compose()`. Substrate-level like `mingl`: no graphics API
dependency, no I/O — pure text processing over `include_str!`-bundled chunks,
usable identically from native (`wgpu`) and browser (`WebGPU`) consumers.

**A chunk** is exactly one WGSL function — or, for the vertex stage, one
entry point plus the struct type it returns — stored as a `.wgsl` file in
its own directory under the repo-root [`shader/`](../../../shader/readme.md)
collection (`shader/<name>/<name>.wgsl`), alongside a `readme.md`
(visualization, manifest fields, and links to related chunks) and a
generated `preview.png`. Bundled chunks: `hash21` (2D-point hash),
`value_noise` (bilinear value noise over `hash21`), `fbm3` (three-octave
fractal Brownian motion over `value_noise`), and `fullscreen_triangle` (the
big-triangle vertex trick: three vertices, no vertex buffer).

**Manifest, not just a file split.** Each chunk opens with a `//@`-prefixed
comment block — the same machine-parsable-attribute convention this
ecosystem's shell playbooks use, just spelled with WGSL's `//` instead of
bash's `#`. A plain `//` comment is for humans only; a `//@ key: value` line
is a header field any tool can pull out with a one-line `grep`/`sed`, e.g.
`sed -n 's|^//@ name: ||p' shader/hash21/hash21.wgsl`. Every chunk declares
`name`, a one-line `description`, `tags` (comma-separated `group:tag` pairs,
blank if none — e.g. `category:noise, technique:fractal`), `depends_on`
(comma-separated, blank if none), and one `export` line per symbol it
exports, giving that symbol's WGSL-syntax signature verbatim (so a reader
never has to leave the header to see how to call it); `fullscreen_triangle`
additionally declares `stage: vertex` since it's an entry point, not a plain
callable function. For example, `shader/value_noise/value_noise.wgsl` opens
with:

```wgsl
//@ name: value_noise
//@ description: Bilinear-interpolated value noise sampled at a 2D point, in [0, 1).
//@ tags: category:noise
//@ depends_on: hash21
//@ export: fn value_noise(p: vec2f) -> f32
```

`compose()`/`try_compose()` actually read and rely on `depends_on`: chunks
can be passed in any order and are still concatenated
dependency-before-dependent; `compose()` panics immediately on a typo'd or
missing dependency or a cycle, naming the offending chunk, while
`try_compose()` is the same sort returning a `ComposeError` instead, for
callers (e.g. a CLI) taking untrusted chunk sets. `CHUNKS` is the
bundled-chunk table — one `ChunkDescriptor` row per chunk, carrying every
manifest field (`name`, `description`, `tags`, `stage`, `depends_on`,
`exports`) as compile-time data plus the full WGSL source — for
enumeration, and `chunk_get( name )` resolves one row by name in O(1),
with no table scan and no manifest parsing; the table is generated at
build time (`build.rs`) straight from those manifests — membership scanned
from the chunk directories, row order taken from the collection-index
table in `shader/readme.md`, the two cross-validated both ways so neither
can drift — so adding a chunk to the collection needs no Rust edit here,
and the drift-guard test (`chunks_table_matches_each_manifest`) holds
every generated row equal to what the parsers read from its manifest. `name_parse`/`description_parse`/
`tags_parse`/`depends_on_parse`/`stage_parse`/`exports_parse` each read
one manifest field directly from arbitrary chunk text, without going
through `compose`. Two tests keep the header
honest against the code it describes:
`depends_on_covers_every_actual_wgsl_call_to_another_chunk` cross-checks
declared dependencies against the chunk's actual WGSL body, and
`export_names_match_a_real_declaration_in_the_wgsl_body` cross-checks every
declared `export` against a real `fn`/`struct` declaration in that same
file — so the manifest can't silently drift out of sync with the body it
sits on top of.

**Importing chunks into an application ( compile-time ).** An application
selects just the bundled chunks it actually uses — not the whole table —
and can define its own chunks beside them, composing both sources as one
set:

```rust
use shader_chunks_core::{ ChunkDescriptor, chunk, dependency_closed, set_compose };

// A locally-defined chunk: same descriptor shape as the bundled rows,
// mirroring the `//@` manifest at the top of its own WGSL file.
const MY_GLOW : ChunkDescriptor = ChunkDescriptor
{
  name : "my_glow",
  description : "App-local glow falloff over the shared noise stack.",
  tags : &[ ( "category", "scene" ) ],
  stage : None,
  depends_on : &[ "value_noise" ],
  exports : &[ "fn my_glow(p: vec2f) -> f32" ],
  wgsl : include_str!( "../shader/my_glow.wgsl" ),
};

const MY_CHUNKS : &[ ChunkDescriptor ] =
&[
  chunk( "hash21" ),      // imported by name — a typo'd name fails the build
  chunk( "value_noise" ),
  MY_GLOW,                // local and imported rows mix freely
];
// A forgotten import fails this assert at build time, not the first frame.
const _ : () = assert!( dependency_closed( MY_CHUNKS ) );

fn shader_source() -> String
{
  // Dependency-before-dependent across both sources, straight from
  // descriptor fields — no manifest parsing at runtime.
  set_compose( MY_CHUNKS )
}
```

`chunk( name )` is a `const fn` returning the descriptor by value, so an
unknown name in `const` position is a compile error; `chunk_get_from(
set, name )` is the same `const` lookup over any caller-supplied set, for
selecting out of a mixed set; `dependency_closed( set )` const-asserts
that every `depends_on` entry is present in the set —
`set_compose`/`set_try_compose`'s success precondition. A crate
defining local chunks keeps each descriptor and its manifest in sync the
same way this crate does: one test per chunk asserting
`manifest_mismatches( &CHUNK )` is empty. Verify the compile-time claims
directly: misspell a `chunk( ... )` name or drop an imported dependency
in a consumer and `cargo check` fails naming the exact line.

**No Rust mirror of any chunk's math** ( no `hash21`/`value_noise`/`fbm3`
Rust ports ). Chunks are a shader-side concept only; a parallel Rust body
would be a second implementation of the same logic — with its own
correctness burden (WGSL's floor-based `fract()` versus Rust's trunc-based
one, for one) — that never runs on the GPU path it mirrors. The manifest,
not a Rust port, is what makes a chunk's interface legible.

**Consumers:** the orrery scene family's browser WebGPU member
[`orrery/webgpu`](../../../examples/orrery/webgpu/readme.md)
imports all four bundled chunks by name and defines its scene-specific
fragment stage (`shader/scene_fragment.wgsl`) as a fifth, local chunk —
its `src/shader_source.rs` is the live model of the import pattern above.

**Design docs:** each mechanism above is documented as a typed doc
instance — registry generation and dependency-ordered composition
(`algorithm/`), selective const import and crate-local chunks
(`pattern/`), dependency closure and descriptor-manifest parity
(`invariant/`) — see [docs/definition/readme.md](docs/definition/readme.md).

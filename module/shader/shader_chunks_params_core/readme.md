# shader_chunks_params_core

**Keywords:** WGSL, Shader Composition, Tunable Parameters, Range Inference

Discovers tunable parameters declared in a shader chunk's manifest header —
a new repeatable `//@ param:` line in the same flat `//@`-prefixed comment
block [`shader_chunks_core`](../shader_chunks_core/readme.md) already reads
for `name`/`description`/`tags`/`depends_on`/`export`. A tunable parameter
is one of 5 kinds: a plain function argument, a compile-time define
directive, a uniform-buffer field, a vertex attribute, or a bound texture.
Substrate-level like `shader_chunks_core`: no graphics API dependency, no
I/O, no execution — pure text processing over raw WGSL strings.

**Grammar:**

```text
//@ param: <name> <kind> <type> [range(min, max)]
```

`<kind>` is one of `argument`/`define`/`uniform`/`attribute`/`texture`;
`<type>` is a WGSL type token copied verbatim from the adjacent real
declaration (`bool`, `u32`, `i32`, `f32`, `vec2f`..`vec4u`, `texture_2d`).
The optional trailing `range(min, max)` always wins when present. When
absent, [`range_infer`](docs/algorithm/001_range_inference_heuristic.md)
resolves one deterministically — a name-substring pattern first (e.g.
`seed` → `[0, 65535]`), a WGSL-type-keyed default second (e.g. bare `u32`
→ `[0, 16]`) — never a random guess. Full grammar and taxonomy:
[`docs/api/001_tunable_parameter_taxonomy.md`](docs/api/001_tunable_parameter_taxonomy.md).

## Usage

```rust
use shader_chunks_params_core::discover;

let wgsl = "\
//@ param: octaves argument u32 range(1, 8)
//@ param: seed define u32

fn fbm( p : vec2f, octaves : u32, seed : u32 ) -> f32 { /* .. */ }
";

let params = discover( wgsl );
assert_eq!( params.len(), 2 );
assert_eq!( params[ 0 ].name, "octaves" ); // declared range: (1.0, 8.0)
assert_eq!( params[ 1 ].name, "seed" );    // inferred range: (0.0, 65535.0)
```

`chunk_discover( &chunk )` is the same parse applied to a
[`shader_chunks_core::ChunkDescriptor`]'s own `.wgsl` field — this crate's
only dependency on `shader_chunks_core`; `discover` itself has none.

## Chunk annotations

Most bundled `shader/*.wgsl` chunks carry `//@ param:` lines today — each
`_preview` browser-demo wrapper's former hardcoded literals are now real
`argument`-kind tunables driving a slider (see
[`shader_chunks_preview_core`](../shader_chunks_preview_core/readme.md)). A
handful of leaf/infrastructure chunks — `hash21`, `value_noise`, `fbm3`,
`fullscreen_triangle` among them — still carry none; this crate remains
independently valuable and fully testable via self-contained fixture WGSL
strings regardless of real-chunk adoption (see `tests/`). Its consumers are
[`shader_chunks_params`](../shader_chunks_params/readme.md) (the `tunables`
CLI over this discovery) and
[`shader_chunks_preview_core`](../shader_chunks_preview_core/readme.md)
(which turns discovered uniform/argument-kind parameters into live browser
sliders).

## Design documentation

- [`docs/api/`](docs/api/readme.md) — the full `//@ param:` grammar, the
  taxonomy types, and every public function's behavior and panic contract.
- [`docs/algorithm/`](docs/algorithm/readme.md) — the range-inference
  heuristic's complete rule table.

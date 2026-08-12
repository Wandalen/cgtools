# shader_chunks

**Keywords:** WGSL, Shader Composition, Shader Manifest, Procedural Generation, Noise

Manifest-driven WGSL shader-chunk composition: small, reusable pieces of WGSL
stored one per file, each carrying a machine-parsable header describing its
interface, composed into a complete shader source at runtime by
`shader_chunks::compose()`. Substrate-level like `mingl`: no graphics API
dependency, no I/O — pure text processing over `include_str!`-bundled chunks,
usable identically from native (`wgpu`) and browser (`WebGPU`) consumers.

**A chunk** is exactly one WGSL function — or, for the vertex stage, one
entry point plus the struct type it returns — stored as a `.wgsl` file under
[`src/chunks/`](src/chunks/). Bundled chunks: `hash21` (2D-point hash),
`value_noise` (bilinear value noise over `hash21`), `fbm3` (three-octave
fractal Brownian motion over `value_noise`), and `fullscreen_triangle` (the
big-triangle vertex trick: three vertices, no vertex buffer).

**Manifest, not just a file split.** Each chunk opens with a `//@`-prefixed
comment block — the same machine-parsable-attribute convention this
ecosystem's shell playbooks use, just spelled with WGSL's `//` instead of
bash's `#`. A plain `//` comment is for humans only; a `//@ key: value` line
is a header field any tool can pull out with a one-line `grep`/`sed`, e.g.
`sed -n 's|^//@ name: ||p' hash21.wgsl`. Every chunk declares `name`, a
one-line `description`, `depends_on` (comma-separated, blank if none), and
one `export` line per symbol it exports, giving that symbol's WGSL-syntax
signature verbatim (so a reader never has to leave the header to see how to
call it); `fullscreen_triangle` additionally declares `stage: vertex` since
it's an entry point, not a plain callable function. For example,
`value_noise.wgsl` opens with:

```wgsl
//@ name: value_noise
//@ description: Bilinear-interpolated value noise sampled at a 2D point, in [0, 1).
//@ depends_on: hash21
//@ export: fn value_noise(p: vec2f) -> f32
```

`compose()` actually reads and relies on `depends_on`: chunks can be passed
to it in any order and are still concatenated dependency-before-dependent,
and a typo'd or missing dependency panics immediately, naming the offending
chunk. Two tests keep the header honest against the code it describes:
`depends_on_covers_every_actual_wgsl_call_to_another_chunk` cross-checks
declared dependencies against the chunk's actual WGSL body, and
`export_names_match_a_real_declaration_in_the_wgsl_body` cross-checks every
declared `export` against a real `fn`/`struct` declaration in that same
file — so the manifest can't silently drift out of sync with the body it
sits on top of.

**No Rust mirror of any chunk's math** ( no `hash21`/`value_noise`/`fbm3`
Rust ports ). Chunks are a shader-side concept only; a parallel Rust body
would be a second implementation of the same logic — with its own
correctness burden (WGSL's floor-based `fract()` versus Rust's trunc-based
one, for one) — that never runs on the GPU path it mirrors. The manifest,
not a Rust port, is what makes a chunk's interface legible.

**Consumers:** the browser WebGPU port
[`scene_script/sun_grid_lines`](../../../examples/scene_script/sun_grid_lines/readme.md)
composes these four chunks ahead of its own scene-specific, fragment-only
WGSL body (`shader/scene_fragment.wgsl`).

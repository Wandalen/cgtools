# Sun Grid Lines, Chunked (wgpu)

**Keywords:** wgpu, Rust, WGSL, Shader Composition, Shader Manifest, Fragment Shader, Procedural Generation, Noise, Offscreen Rendering

A variant of [`sun_grid_lines`](../sun_grid_lines/readme.md) that renders the
identical image but assembles its shader from **chunks**: small, reusable
pieces of WGSL this example shares with its siblings, instead of duplicating
them inline into one `scene.wgsl`. Each chunk is exactly one function — or,
for the vertex stage, one entry point plus the struct type it returns —
stored as a `.wgsl` file whose leading comment block is a **manifest**
describing its interface: `hash21`, `value_noise`, `fbm3`, and
`fullscreen_triangle`. At startup, `src/shader_chunks::compose()` reads every
chunk's manifest header, topologically sorts the chunks by their declared
dependencies, and concatenates their WGSL bodies in that order — before this
example's own fragment body (`shaders/scene_fragment.wgsl`) is appended as
the final, non-reusable "program" that consumes them.

That duplication is real, not hypothetical: the fullscreen-triangle trick and
the noise functions are hand-copied verbatim across this example, its
[Vulkan-backend sibling](../sun_grid_lines_vulkan/readme.md), the
[browser WebGPU port](../../scene_script/sun_grid_lines/readme.md), and
re-derived by hand in GLSL for the
[WebGL2 original](../../minwebgl/sun_grid_lines/readme.md) — four independent
copies of the same two ideas. This example doesn't fix that duplication
workspace-wide; it demonstrates, for one example, what removing it locally
looks like.

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

`src/shader_chunks.rs` is a **local, example-scoped module**, not a
published crate — promoting this pattern to a shared workspace crate (e.g.
`module/min/shader_chunks`, substrate-level like `mingl`) so the other
`sun_grid_lines` variants could depend on it too is a separate, deferred
decision, not made here. Deliberately out of scope for this example: no
Rust-side reimplementation of any chunk's math. An earlier draft of this
example ported `hash21`/`value_noise`/`fbm3` into parallel Rust functions;
that added a second implementation of the same logic — with its own
correctness burden (WGSL's floor-based `fract()` versus Rust's trunc-based
one, for one) — that never ran on the GPU path it was meant to mirror. This
version keeps chunks entirely on the shader side: the manifest, not a Rust
mirror of the body, is what makes a chunk's interface legible.

There is no windowing anywhere in this workspace (no `winit`, no
`wgpu::Surface`), so this example follows the same pattern as
[`hello_triangle`](../hello_triangle/readme.md) and
[`grid_render`](../grid_render/readme.md): render once into an offscreen
texture, copy it into a `MAP_READ` buffer, and save the result as
`-sun_grid_lines_chunked.png`. No live loop, no animation, no keyboard
interactivity — `node_count = 4` is baked into a single fixed uniform buffer
at buffer-creation time, to show off the orbiting-node parameterization in
the one frame this example produces.

**No multi-pass bloom**, for the same reason as the other native/WebGPU
ports: no post-processing library exists for `minwgpu` in this workspace.
Glow uses the shader's analytic radial-falloff / `exp()` terms.

![image](showcase.webp)

**[How to run](../../how_to_run.md)**

**References:**

* [wgpu Documentation]
* [WGSL Specification]
* [inigo quilez — fbm]

[wgpu Documentation]: https://docs.rs/wgpu/latest/wgpu/
[WGSL Specification]: https://www.w3.org/TR/WGSL/
[inigo quilez — fbm]: https://iquilezles.org/articles/fbm/

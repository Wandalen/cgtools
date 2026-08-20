# shader_chunks_preview_core

**Keywords:** WGSL, Shader Composition, Live Preview, WebGPU, Uniform Buffer

Builds a self-contained **preview bundle** — composed WGSL plus a slider
parameter list — from one target shader chunk. The bundle is the whole
interface between the native
[`shader_chunks_preview`](../shader_chunks_preview/readme.md) CLI (which
builds, naga-validates, and serializes it) and the
[`shader_chunks_preview_web`](../shader_chunks_preview_web/readme.md)
browser runner (which deserializes it and renders: one slider per
parameter, one uniform buffer laid out by the convention below). Pure
text processing over [`shader_chunks_core`](../shader_chunks_core/readme.md)'s
manifests and
[`shader_chunks_params_core`](../shader_chunks_params_core/readme.md)'s
`//@ param:` discovery — no I/O, no graphics API, wasm-clean (the browser
runner depends on this crate directly, on the wasm32 target).

**Two target modes**, selected from the target's own manifest:

```text
Fragment chunk ( //@ stage: fragment )
  must export fs_main; its //@ param: uniform f32 lines become the sliders

Value chunk ( any fn NAME(p: vec2f) -> T export, T in f32/vec2f/vec3f )
  a fragment harness is synthesized around the export, with one synthesized
  preview_scale slider, plus a world-space reference grid ( unit-spaced
  minor lines, emphasized axes at the origin ) overlaid on every shape so
  scale/center stay legible. A category:sdf-tagged chunk's f32 shape gets a
  filled-inside / distance-banded-outside / crisp-zero-isoline treatment
  with a stationary sample point, instead of a raw value blurred by a flat
  [0, 1] clamp and panned out of frame by an unbounded time drift. Every
  other shape/tag combination keeps the original convention: aspect-
  corrected, time-drifting, raw value written and clamped to [0, 1] by the
  render target — grayscale (f32), blue-padded 2-channel (vec2f), or direct
  RGB (vec3f).
```

Composed WGSL is banner-commented per section
(`// ==== dependency chunk: NAME ====`, `// ==== previewing: NAME ... ====`,
`// ==== auto-generated preview harness ... ====`) so the concatenated text
— what `render` and the live browser editor both show — makes clear which
part is a dependency, which part is the chunk under preview, and which
part (if any) has no hand-written counterpart.

**Uniform layout convention** (what the browser runner writes, and what a
fragment-mode chunk's own `struct Params` must therefore declare):
`time : f32` first, then each `//@ param:` uniform as `f32` in declaration
order, then `resolution : vec4f` (`.xy` = physical pixels) — WGSL's own
struct rules place `resolution` at the next 16-byte boundary, and
[`resolution_index`] computes exactly that boundary from the parameter
count so both the CLI's validation and the runner's buffer-writing agree
on the same layout without duplicating the arithmetic.

## Usage

```rust
use shader_chunks_preview_core::bundle_build;

let chunk = shader_chunks_core::chunk_get( "fbm3" ).unwrap();
let bundle = bundle_build( chunk.wgsl ).unwrap();
assert_eq!( bundle.target, "fbm3" );
assert!( !bundle.parameters.is_empty() ); // value chunk: synthesized preview_scale slider
```

## Errors

[`PreviewError`] covers every reason a chunk can fail to preview:
`UnknownChunk` (a dependency not bundled), `Unpreviewable` (missing
manifest lines, no previewable export, or a fragment chunk missing
`fs_main`/any `//@ param:` uniform), `UnsupportedParam` (a `//@ param:`
outside the `uniform f32` convention — including a value chunk that
declares its own params, since the synthesized harness owns the uniform
struct instead), and `Compose` (the assembled chunk set fails
[`shader_chunks_core`] composition). None of the 4 bundled chunks are
fragment-mode today — every current preview goes through the
value-chunk/synthesized-harness path; the fragment-chunk path is
exercised by this crate's own fixture-based tests.

## Design documentation

- [`docs/algorithm/`](docs/algorithm/readme.md) — the value-function
  shape detection heuristic's complete decision table (which export
  shapes are previewable, how candidate selection picks among them, and
  how each shape is written to the render target).

# Command :: 8. render

### Description

Renders one static frame of a chunk's preview bundle on a headless GPU
and writes it as a PNG — a bundled chunk by `name`, or a local file via
`file::`. Target resolution and naga validation are byte-identical to
[`.preview`](../../../../shader_chunks_preview/docs/cli/command/01_preview.md)'s (the command reuses
`shader_chunks_preview::bundle_prepare` verbatim), so anything
previewable is renderable and vice versa. Every bundle parameter takes
its initial (slider-start) value and the bundle's `time` uniform is
frozen at `time::` — the written image is exactly what the browser
preview shows before anyone touches a slider, at that instant.

-- **Parameters:** name, file, out, size, time
-- **Exit Codes:** 0 (success — PNG written, summary printed) | 1 (no
   target or both targets given; `name` does not resolve against
   `shader_chunks_core::CHUNKS`; composed WGSL fails naga
   parse/validation; `size::` malformed or has a zero side) | 2 (the
   `file::` target could not be read; no usable headless GPU
   adapter/device; the GPU rejected the render; the PNG could not be
   written)
-- **Modes:** (none)

### Syntax
```bash
shader_chunks render [<name>] [file::<path>] [out::<path>] [size::<n>|<w>x<h>] [time::<seconds>]
```

### Parameters

| Parameter | Type | Default | Required | Purpose |
|-----------|------|---------|----------|---------|
| `name` | [`ChunkName`](../../../../shader_chunks_query/docs/cli/param/01_name.md) | — | Exactly one of `name`/`file::` | The bundled chunk to render |
| `file` | [`String`](../../../../shader_chunks_preview/docs/cli/param/01_file.md) | — | Exactly one of `name`/`file::` | A local `.wgsl` chunk file to render instead of a bundled name |
| `out` | [`String`](../param/01_out.md) | `<target>.png` | No | Output PNG path |
| `size` | [`String`](../param/02_size.md) | `256` | No | Output size: `<n>` (square) or `<width>x<height>`, each side ≥ 1 |
| `time` | [`Float`](../param/03_time.md) | `0` | No | Value of the bundle's `time` uniform for this frame |

### Examples
```bash
shader_chunks render fbm3
# wrote fbm3.png (256x256 px, naga-validated)
# target: fbm3
# time: 0
# parameters at defaults:
#   preview_scale = 8

shader_chunks render fbm3 out::fbm3_far.png size::512 time::2.5
# same pipeline, 512x512, the drift harness advanced 2.5 seconds

shader_chunks render file::-my_harness.wgsl size::128x64
# renders a local WGSL chunk file at a non-square size

shader_chunks render
# error: render needs exactly one target: a chunk name (see `list`) or `file::<path>`, exit 1

shader_chunks render bogus_chunk
# error: unknown chunk: `bogus_chunk` (see `shader_chunks list` for valid names), exit 1

shader_chunks render fbm3 size::0
# error: invalid `size` value: `0` (allowed: `<n>` or `<width>x<height>`, each side at least 1), exit 1
```

### Notes
- `name` and `file::` are mutually exclusive and jointly required — giving
  both, or neither, fails with exit 1 before any chunk lookup or file
  read is attempted; both arms are subprocess-tested.
- Validation runs before any GPU work and before any write: a chunk that
  fails naga parse or validation leaves the `out::` path untouched, and
  a failed render never writes a partial PNG.
- Chunk coverage equals [`.preview`](../../../../shader_chunks_preview/docs/cli/command/01_preview.md)'s exactly: a fragment
  chunk exporting `fs_main` with `//@ param:` uniforms, or a value chunk
  exporting `fn NAME(p: vec2f) -> f32` (synthesized grayscale harness).
  A chunk outside those shapes (e.g. `hash22`, returning `vec2f`) is
  rejected as not previewable, exit 1 — `file::` with a hand-written
  fragment harness is the escape hatch.
- The render target is `Rgba8Unorm`, not `Rgba8UnormSrgb` — chunks write
  display-ready values (the collection's `srgb` chunk exists precisely
  because encoding is the author's explicit move), so an sRGB target
  would double-encode them. See
  [`shader_chunks_render_core`](../../../../shader_chunks_render_core/readme.md).
- Like `.preview`, `.render` has a real side effect beyond stdout: it
  unconditionally (re)writes the `out::` path on success. Unlike
  `.preview` it spawns no subprocess, needs no browser, and touches no
  other crate's directory — see [Render](../command_group/01_render.md)'s
  Invariants and its "Why NOT Merge Into Preview" section.
- Output format: [`plain_text`](../../../../shader_chunks_compose/docs/cli/format/01_plain_text.md) — the
  summary is printed as-is, same as `.preview`'s summary; the image
  itself goes to the filesystem, never to stdout.

### Related Commands

| # | Command | Relationship |
|---|---------|--------------|
| 1 | [`.preview`](../../../../shader_chunks_preview/docs/cli/command/01_preview.md) | Same bundle building and validation lineage, rendered live in the browser instead of to a file |
| 2 | [`.compose`](../../../../shader_chunks_compose/docs/cli/command/01_compose.md) | Same WGSL composition lineage, but prints text and renders nothing |
| 3 | [`.tunables`](../../../../shader_chunks_params/docs/cli/command/01_tunables.md) | Lists the `//@ param:` declarations whose default values `.render` bakes into the frame |

### Referenced User Stories

*(None — this project deliberately omits the `user_story/` collection at
this CLI's scale; see [`docs/cli/readme.md` § Scope
Decisions](../readme.md#scope-decisions).)*

---

**Category:** chunk
**Complexity:** 3
**API Requirement:** None
**Idempotent:** Yes
**Risk Level:** Low

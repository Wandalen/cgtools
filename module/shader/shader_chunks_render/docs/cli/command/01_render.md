# Command :: 8. render

### Description

Renders one static frame of a chunk's preview bundle on a headless GPU
and writes it as a PNG — a bundled chunk by `name`, or a local file via
`file::`. Target resolution and naga validation are byte-identical to
[`.preview`](../../../../shader_chunks_preview/docs/cli/command/01_preview.md)'s (the command reuses
`shader_chunks_preview::bundle_prepare` verbatim), so anything
previewable is renderable and vice versa. Every bundle parameter takes
its initial (slider-start) value, unless overridden via `set::`, and the
bundle's `time` uniform is frozen at `time::` — the written image is
exactly what the browser preview shows before anyone touches a slider
(or, with `set::` applied, mid-drag), at that instant. With `all::1`,
sweeps every entry in `shader_chunks_core::CHUNKS` in one pass instead of
resolving a single target, writing `<out>/<name>.png` per chunk; a chunk
outside the previewable shapes is skipped, not failed, and one chunk's
render/validation failure does not stop the rest of the batch.

-- **Parameters:** name, file, out, size, time, set, all
-- **Exit Codes:** 0 (success — PNG(s) written, summary printed) | 1 (no
   target or both targets given; `name` does not resolve against
   `shader_chunks_core::CHUNKS`; composed WGSL fails naga
   parse/validation; `size::` malformed or has a zero side; a `set::`
   element is malformed or non-finite; a `set::` property matches no
   declared parameter; `all::1` combined with `name`/`file::`/`set::`; at
   least one chunk failed during an `all::1` sweep) | 2 (the `file::`
   target could not be read; no usable headless GPU adapter/device; the
   GPU rejected the render; the PNG could not be written; the `all::1`
   output directory could not be created)
-- **Modes:** (none)

### Syntax
```bash
shader_chunks render [<name>] [file::<path>] [out::<path>] [size::<n>|<w>x<h>] [time::<seconds>] [set::<property>:<value>,...] [all::1]
```

### Parameters

| Parameter | Type | Default | Required | Purpose |
|-----------|------|---------|----------|---------|
| `name` | [`ChunkName`](../../../../shader_chunks_query/docs/cli/param/01_name.md) | — | Exactly one of `name`/`file::` (unless `all::1`) | The bundled chunk to render |
| `file` | [`String`](../../../../shader_chunks_preview/docs/cli/param/01_file.md) | — | Exactly one of `name`/`file::` (unless `all::1`) | A local `.wgsl` chunk file to render instead of a bundled name |
| `out` | [`String`](../param/01_out.md) | `<target>.png` | No | Output PNG path (or output DIRECTORY under `all::1`) |
| `size` | [`String`](../param/02_size.md) | `256` | No | Output size: `<n>` (square) or `<width>x<height>`, each side ≥ 1 |
| `time` | [`Float`](../param/03_time.md) | `0` | No | Value of the bundle's `time` uniform for this frame |
| `set` | [`ParameterOverride`](../type/02_parameter_override.md) list | none (bundle defaults) | No | Comma-separated `property:value` overrides for the bundle's declared parameters |
| `all` | [`Switch`](../../../../shader_chunks_query/docs/cli/type/07_switch.md) | `false` | No | Render every previewable chunk instead of one target |

### Examples
```bash
shader_chunks render fbm3
# wrote fbm3.png (256x256 px, naga-validated)
# target: fbm3
# time: 0
# parameters:
#   lacunarity = 2
#   gain = 0.5
#   preview_scale = 8

shader_chunks render fbm3 out::fbm3_far.png size::512 time::2.5
# same pipeline, 512x512, the drift harness advanced 2.5 seconds

shader_chunks render file::-my_harness.wgsl size::128x64
# renders a local WGSL chunk file at a non-square size

shader_chunks render fbm3 set::lacunarity:2.5,gain:0.75
# wrote fbm3.png (256x256 px, naga-validated)
# target: fbm3
# time: 0
# parameters:
#   lacunarity = 2.5
#   gain = 0.75
#   preview_scale = 8

shader_chunks render
# error: render needs exactly one target: a chunk name (see `list`) or `file::<path>`, exit 1

shader_chunks render bogus_chunk
# error: unknown chunk: `bogus_chunk` (see `shader_chunks list` for valid names), exit 1

shader_chunks render fbm3 size::0
# error: invalid `size` value: `0` (allowed: `<n>` or `<width>x<height>`, each side at least 1), exit 1

shader_chunks render fbm3 set::bogus:1.0
# error: unknown parameter: `bogus` (valid parameters: lacunarity, gain, preview_scale), exit 1

shader_chunks render all::1 out::renders/ size::128
# fbm3: wrote renders/fbm3.png
# fullscreen_triangle: skipped (chunk `fullscreen_triangle` is not previewable ...)
# hash21: wrote renders/hash21.png
# ... (one line per registry entry; exact count and names grow with the collection)
# <n> chunks: <n - 1> rendered, 1 skipped, 0 failed

shader_chunks render fbm3 all::1
# error: render `all::1` renders every chunk and cannot be combined with a target (`name`/`file::`) or `set::`, exit 1
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
- `set::` lets one frame capture any parameter combination, not just the
  defaults: parsing (`overrides_parse`) and identity resolution against
  the live bundle (`overrides_apply`) are independent steps, so a
  malformed element (exit 1, before target resolution's cost is paid)
  and an unrecognized property name (exit 1, after the bundle is built)
  fail with distinct, specific messages. Overrides are baked in as-is —
  never clamped to a parameter's declared `min`/`max` — and a later
  element overriding the same property as an earlier one wins. See
  [`set`](../param/04_set.md).
- `all::1` sweeps every entry in `shader_chunks_core::CHUNKS` instead of
  resolving one target — mutually exclusive with `name`, `file::`, and
  `set::` (a single override list can't cleanly apply across chunks with
  different declared parameters). `out::` is read as a DIRECTORY under
  `all::1`, created if it doesn't already exist; each chunk writes
  `<dir>/<name>.png`. A chunk outside the previewable shapes is skipped,
  not failed, and does not stop the sweep; any other per-chunk error
  (naga validation, GPU, io) is also non-stopping but flips the overall
  exit code to 1 once every chunk has been attempted. See
  [`all`](../param/05_all.md).

### Related Commands

| # | Command | Relationship |
|---|---------|--------------|
| 1 | [`.preview`](../../../../shader_chunks_preview/docs/cli/command/01_preview.md) | Same bundle building and validation lineage, rendered live in the browser instead of to a file |
| 2 | [`.compose`](../../../../shader_chunks_compose/docs/cli/command/01_compose.md) | Same WGSL composition lineage, but prints text and renders nothing |
| 3 | [`.tunables`](../../../../shader_chunks_params/docs/cli/command/01_tunables.md) | Lists the `//@ param:` declarations whose property names `set::` overrides target, and whose default values `.render` bakes in when `set::` is omitted |

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

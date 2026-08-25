# Command :: 7. preview

### Description

Builds a live browser preview bundle for one WGSL chunk — a bundled
chunk by `name`, or a local file via `file::` — naga-validates the
composed WGSL exactly as `wgpu` would parse it, writes the bundle as
`-preview.json` into the `shader_chunks_preview_web` runner crate, and by
default launches that runner in the browser via the repo's shared
`action/browser_serve` script. `serve::0` stops after writing and prints
a summary instead — target, composed size, and the sliders wired to the
chunk's `//@ param:` uniforms — which is also what makes the command
testable end-to-end without a browser.

-- **Parameters:** name, file, serve
-- **Exit Codes:** 0 (success — bundle written, and served unless
   `serve::0`) | 1 (no target or both targets given; `name` does not
   resolve against `shader_chunks_core::CHUNKS`; composed WGSL fails naga
   parse/validation) | 2 (the `file::` target could not be read, or
   launching the browser dev server failed)
-- **Modes:** `serve::1` (default) — build, validate, write, then hand
   off to the browser, blocking until the dev server stops. `serve::0` —
   build, validate, write, then print the summary and return.

### Syntax
```bash
shader_chunks preview [<name>] [file::<path>] [serve::<0|1>]
```

### Parameters

| Parameter | Type | Default | Required | Purpose |
|-----------|------|---------|----------|---------|
| `name` | [`ChunkName`](../../../../shader_chunks_query/docs/cli/param/01_name.md) | — | Exactly one of `name`/`file::` | The bundled chunk to preview |
| `file` | [`String`](../param/01_file.md) | — | Exactly one of `name`/`file::` | A local `.wgsl` chunk file to preview instead of a bundled name |
| `serve` | [`Switch`](../param/02_serve.md) | `true` | No | Launch the browser dev server after writing the bundle |

### Examples
```bash
shader_chunks preview fbm3
# wrote .../shader_chunks_preview_web/-preview.json (... bytes wgsl, naga-validated)
# target: fbm3
# sliders:
#   preview_scale  1..32  start 8
# (then blocks, serving the runner in the browser)

shader_chunks preview fbm3 serve::0
# same summary, printed once, no browser hand-off — process exits after printing

shader_chunks preview file::shader/fbm3/fbm3.wgsl serve::0
# previews a local WGSL file instead of a bundled chunk name

shader_chunks preview
# error: preview needs exactly one target: a chunk name (see `list`) or `file::<path>`, exit 1

shader_chunks preview bogus_chunk
# error: unknown chunk: `bogus_chunk` (see `shader_chunks list` for valid names), exit 1
```

### Notes
- `name` and `file::` are mutually exclusive and jointly required — giving
  both, or neither, fails with exit 1 before any chunk lookup or file
  read is attempted.
- Validation runs before any write: a chunk that fails naga parse or
  validation leaves `-preview.json` untouched.
- Unlike every other command in this CLI, `.preview` has real side
  effects beyond stdout: it unconditionally (re)writes `-preview.json` in
  `shader_chunks_preview_web`'s crate directory on success, and with the
  default `serve::1` also spawns a blocking subprocess
  (`action/browser_serve`) that launches a local dev server. See
  [Preview](../command_group/01_preview.md)'s Invariants for the exact
  boundary — this is the deliberate reason `.preview` is its own group
  rather than folded into [Compose](../../../../shader_chunks_compose/docs/cli/command_group/01_compose.md).
- Sliders come from the target's own `//@ param:` declarations (a
  fragment chunk exporting `fs_main`) or, for a value chunk exporting
  `fn NAME(p: vec2f) -> f32`, a single synthesized `preview_scale`
  slider — see
  [`shader_chunks_preview_core`](../../../../shader_chunks_preview_core/readme.md)
  for the two-mode bundle-building rule.
- Output format: [`plain_text`](../../../../shader_chunks_compose/docs/cli/format/01_plain_text.md) — the
  summary is printed as-is, same as `.compose`'s WGSL output, never
  through the tabular `data_fmt` pipeline.

### Related Commands

| # | Command | Relationship |
|---|---------|--------------|
| 1 | [`.tunables`](../../../../shader_chunks_params/docs/cli/command/01_tunables.md) | Lists the same `//@ param:` declarations `.preview` wires to sliders, without building or serving anything |
| 2 | [`.compose`](../../../../shader_chunks_compose/docs/cli/command/01_compose.md) | Same WGSL composition and validation lineage, but prints text and writes nothing |
| 3 | [`.render`](../../../../shader_chunks_render/docs/cli/command/01_render.md) | The identical bundle frozen to a static PNG file instead of served live |

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

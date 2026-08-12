# shader

Repo-root collection of reusable WGSL **shader chunks** — small, composable
pieces of shader source, one per directory, bundled at compile time and
composed by [`shader_chunks_core`](../module/shader/shader_chunks_core/readme.md)
and inspected/composed from the terminal by the
[`shader_chunks`](../module/shader/shader_chunks/readme.md) (`sch`) CLI.

Each chunk lives in its own directory: `shader/<name>/<name>.wgsl` (the
chunk's WGSL source, opening with a `//@`-prefixed manifest header) plus a
`readme.md` (visualization, parameters, and links to related chunks) and a
`preview.png` (a generated visualization of what the chunk actually
produces). See any chunk's own `readme.md` for the manifest-field
conventions (`name`/`description`/`tags`/`stage`/`depends_on`/`export`).

| Chunk | Responsibility | Depends On |
|-------|-----------------|------------|
| [hash21/](hash21/readme.md) | Hash a 2D point to a single pseudo-random value | — |
| [value_noise/](value_noise/readme.md) | Bilinear-interpolated smooth noise over `hash21` | `hash21` |
| [fbm3/](fbm3/readme.md) | 3-octave fractal Brownian motion over `value_noise` | `value_noise` |
| [fullscreen_triangle/](fullscreen_triangle/readme.md) | Big-triangle vertex stage covering the viewport | — |

Dependency order (also see `sch tree`, which derives this live from each
chunk's `//@ depends_on:` header rather than from this table):

```
fbm3 → value_noise → hash21
fullscreen_triangle   (standalone; no dependencies)
```

All four chunks are consumed together by the orrery scene family's browser
WebGPU member, [`examples/orrery/webgpu`](../examples/orrery/webgpu/readme.md),
which composes them ahead of its own scene-specific fragment body.

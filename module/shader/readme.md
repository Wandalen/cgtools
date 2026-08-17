# shader

**Keywords:** WGSL, Shader Composition, CLI, Query Engine, Live Preview, Headless Render

The `shader_chunks` crate family: manifest-driven WGSL shader-chunk
composition ([`shader_chunks_core`](shader_chunks_core/readme.md)) plus
five independent terminal utilities built over it — query, compose,
params, preview, render — aggregated under one CLI
([`shader_chunks`](shader_chunks/readme.md)) but each also runnable
standalone. Every utility's argument wiring, dispatch, and help rendering
share one layer ([`shader_chunks_cli_core`](shader_chunks_cli_core/readme.md));
every utility with real query/compose/discovery/preview logic separates
that logic into its own `_core` engine crate, except
[`shader_chunks_compose`](shader_chunks_compose/readme.md), which is thin
enough that `shader_chunks_core` itself serves as its core.

### Responsibility Table

| Crate | Responsibility |
|---|---|
| [`shader_chunks_core/`](shader_chunks_core/readme.md) | Manifest-driven WGSL chunk registry and dependency-ordered composition |
| [`shader_chunks_params_core/`](shader_chunks_params_core/readme.md) | Discovers `//@ param:` tunable parameters in a chunk's manifest |
| [`shader_chunks_query_core/`](shader_chunks_query_core/readme.md) | Filter/project/sort/page/render query engine over bundled chunks |
| [`shader_chunks_query/`](shader_chunks_query/readme.md) | CLI wiring for `list`/`get`/`tags`/`tree` over the query engine |
| [`shader_chunks_compose/`](shader_chunks_compose/readme.md) | CLI and logic for the `compose` command (no separate `_core`) |
| [`shader_chunks_params/`](shader_chunks_params/readme.md) | CLI wiring for `tunables` over the params discovery engine |
| [`shader_chunks_preview_core/`](shader_chunks_preview_core/readme.md) | Builds a composed, slider-annotated preview bundle from one chunk |
| [`shader_chunks_preview/`](shader_chunks_preview/readme.md) | CLI wiring for `preview`: builds, naga-validates, writes, serves |
| [`shader_chunks_preview_web/`](shader_chunks_preview_web/readme.md) | wasm32-only WebGPU browser runner rendering a written preview bundle |
| [`shader_chunks_render_core/`](shader_chunks_render_core/readme.md) | Renders a preview bundle to raw RGBA pixels on a headless GPU |
| [`shader_chunks_render/`](shader_chunks_render/readme.md) | CLI wiring for `render`: builds, naga-validates, renders, writes a PNG |
| [`shader_chunks_cli_core/`](shader_chunks_cli_core/readme.md) | Shared `unilang` dispatch, help rendering, and exit-code plumbing |
| [`shader_chunks/`](shader_chunks/readme.md) | Aggregates every utility's commands under one `shader_chunks`/`sch` binary |

Not to be confused with the repo-root [`shader/`](../../shader/readme.md)
collection — the raw `.wgsl` chunk files
[`shader_chunks_core`](shader_chunks_core/readme.md) bundles at build
time. This directory holds the Rust crates that read, query, compose, and
preview that collection; it contains no `.wgsl` source of its own.

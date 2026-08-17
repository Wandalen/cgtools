# shader_chunks_render

**Keywords:** WGSL, Headless Rendering, PNG, CLI, naga, WebGPU

Render utility CLI: the single `render` command, reusing
[`shader_chunks_preview`](../shader_chunks_preview/readme.md)'s
`bundle_prepare` — the same target resolution (bundled name or local
`file::`) and the same naga validation the live preview runs — then
rendering one frame of the bundle on a headless GPU via
[`shader_chunks_render_core`](../shader_chunks_render_core/readme.md)
and writing it as a PNG. Exposes its command set, help group, and help
examples as data — parameterized by binary name — so the
[`shader_chunks`](../shader_chunks/readme.md) aggregator folds it in
unchanged, while [`run`] serves the same command as the standalone
`shader_chunks_render` binary.

**Pipeline** (`render`):

```text
render <name> | file::<path>   [out::<path>] [size::<n>|<w>x<h>] [time::<seconds>] [set::<property>:<value>,...]
  -> size_parse( size )                        # `256` (square) or `128x64`; each side >= 1
  -> shader_chunks_preview::bundle_prepare     # compose + naga-validate, BEFORE any GPU work
  -> overrides_parse( set ) + overrides_apply  # replace named parameters' defaults; later wins
  -> shader_chunks_render_core::render         # one headless frame, params at their resolved values
  -> image::save_buffer( out, RGBA8 )          # default out: <target>.png in the cwd
  -> summary                                   # "wrote ... (WxH px, naga-validated)" + baked values
```

The written frame is exactly what the browser preview shows before anyone
touches a slider — every bundle parameter takes its initial value unless
overridden via `set::`, and `time::` freezes the preview's time uniform at
one instant (default `0`).
`name` resolves against the bundled registry; `file::<path>` renders any
local `.wgsl` chunk file instead, which is how a custom hand-written
harness (or a chunk still under development) gets a static image. Exactly
one of the two is required; giving both or neither fails with exit `1`
before either is resolved.

## Structure

| Path | Responsibility |
|---|---|
| `src/` | `render` command wiring — target resolution, `size_parse`, PNG write |
| `tests/` | [`render_cli_test.rs`](tests/render_cli_test.rs) plus the [`docs/cli/`](docs/cli/readme.md) specification mirror at [`tests/docs/cli/`](tests/docs/cli/readme.md) |
| `docs/` | CLI documentation — see [`docs/cli/`](docs/cli/readme.md) for the `render` command, `out`/`size`/`time`/`set` params, and the `Float`/`ParameterOverride` types |
| `Cargo.toml` | Crate manifest and dependencies |

## Usage

```sh
cargo run -p shader_chunks_render -- render fbm3                # writes ./fbm3.png, 256x256
cargo run -p shader_chunks_render -- render fbm3 size::512 time::2.5 out::fbm3_far.png
cargo run -p shader_chunks_render -- render file::-my_harness.wgsl size::128x64
cargo run -p shader_chunks_render -- render fbm3 set::lacunarity:2.5,gain:0.75
```

```rust
use shader_chunks_preview::PreviewTarget;
use shader_chunks_render::{ render_to_png, size_parse };

assert_eq!( size_parse( "128x64" ).unwrap(), ( 128, 64 ) );
let out = std::env::temp_dir().join( "fbm3.png" );
let overrides = vec![ ( "gain".to_string(), 0.9 ) ];
let summary = render_to_png( &PreviewTarget::Name( "fbm3".to_string() ), ( 64, 64 ), 0.0, &overrides, &out ).unwrap();
assert!( summary.contains( "naga-validated" ) );
assert!( summary.contains( "gain = 0.9" ) );
```

## Errors

[`RenderCliError`] maps to exit codes by kind: a bad target, a bad
`size::`, or a shader that fails naga (all caller-fixable) exit `1`; a
missing GPU adapter, a device validation failure, or an unwritable
`out::` path (environmental) exit `2`. Validation runs before any GPU
work and before any write — a chunk that fails naga never produces a
partial or stale PNG. Coverage of a chunk is the same as the live
preview's: a fragment chunk exporting `fs_main`, or a value chunk
exporting `fn NAME(p: vec2f) -> f32` — anything else is rejected as
not previewable (exit `1`), same as `.preview` would; `file::` with a
hand-written fragment harness is the escape hatch for chunks outside
those shapes.

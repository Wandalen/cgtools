# shader_chunks_preview

**Keywords:** WGSL, Shader Composition, Live Preview, CLI, naga, WebGPU

Preview utility CLI: the single `preview` command, building and
naga-validating a
[`shader_chunks_preview_core`](../shader_chunks_preview_core/readme.md)
bundle, writing it for the
[`shader_chunks_preview_web`](../shader_chunks_preview_web/readme.md)
browser runner, and (by default) blocking on the browser dev-server
hand-off so the result shows up live. Exposes its command set, help
group, and help examples as data — parameterized by binary name — so the
[`shader_chunks`](../shader_chunks/readme.md) aggregator folds it in
unchanged, while [`run`] serves the same command as the standalone
`shader_chunks_preview` binary.

**Pipeline** (`preview`):

```text
preview <name> | file::<path>   [serve::true|false]  (exactly one of name/file required)
  -> bundle_prepare( target )
       -> shader_chunks_preview_core::bundle_build     // compose the WGSL + slider list
       -> naga::front::wgsl::parse_str + naga::valid::Validator   // BEFORE any write
  -> bundle_write( bundle, web_crate_dir() )            // writes -preview.json
  -> summary( bundle, written_to )                      // "wrote ... (naga-validated)" + sliders
  -> serve::true (default): action/browser_serve subprocess, blocks
     serve::false: prints the summary, returns
```

`name` resolves against the bundled registry; `file::<path>` reads any
local `.wgsl` file instead — bundled or not, which is what makes
`.preview` usable on a chunk still under development. Exactly one of the
two is required; giving both or neither fails with exit `1` before either
is resolved. [`web_crate_dir`] locates
[`shader_chunks_preview_web`](../shader_chunks_preview_web/readme.md) via
`env!("CARGO_MANIFEST_DIR")` — a compile-time path, not a runtime
environment read, so it resolves correctly regardless of the process's
current working directory.

## Structure

| Path | Responsibility |
|---|---|
| `src/` | `preview` command wiring — bundle build, naga validation, dev-server hand-off |
| `tests/` | [`preview_cli_test.rs`](tests/preview_cli_test.rs) plus the [`docs/cli/`](docs/cli/readme.md) specification mirror at [`tests/docs/cli/`](tests/docs/cli/readme.md) |
| `docs/` | CLI documentation — see [`docs/cli/`](docs/cli/readme.md) for the `preview` command and `file`/`serve` params |
| `Cargo.toml` | Crate manifest and dependencies |

## Usage

```sh
cargo run -p shader_chunks_preview -- preview fbm3            # builds, validates, serves live
cargo run -p shader_chunks_preview -- preview fbm3 serve::0   # builds, validates, writes, prints summary, exits
cargo run -p shader_chunks_preview -- preview file::shader/fbm3/fbm3.wgsl serve::0
```

```rust
use shader_chunks_preview::{ bundle_prepare, preview, PreviewTarget };

let bundle = bundle_prepare( &PreviewTarget::Name( "fbm3".to_string() ) ).unwrap();
assert!( !bundle.parameters.is_empty() );

let summary = preview( &PreviewTarget::Name( "fbm3".to_string() ), false ).unwrap();
assert!( summary.contains( "naga-validated" ) );
```

## Errors

[`PreviewCliError`] maps to exit codes by kind: `UnknownChunk` / `Preview`
/ `Validation` (validation-style, caller-fixable) exit `1`; `Io` / `Serve`
(environmental — an unreadable file, a dev-server failure) exit `2`.
Validation always runs before any file is written — a chunk that fails
naga parse/validation never produces a stale or partial `-preview.json`.

**Disclosed gap:** no test covers giving *both* `name` and `file::`
simultaneously — only the "neither given" arm of the mutual-exclusivity
check is exercised. See
[`tests/docs/cli/command/cmd_001_preview.md`](tests/docs/cli/command/cmd_001_preview.md)
for the full test-coverage account.

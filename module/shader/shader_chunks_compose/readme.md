# shader_chunks_compose

**Keywords:** WGSL, Shader Composition, CLI, Dependency Resolution

Compose utility CLI: the single `compose` command over
[`shader_chunks_core`](../shader_chunks_core/readme.md)'s own resolution
(`set_resolve`) and composition (`set_try_compose`). This utility
deliberately has **no** `_core` crate of its own — `shader_chunks_core`
*is* its core, unlike
[`shader_chunks_query`](../shader_chunks_query/readme.md) or
[`shader_chunks_params`](../shader_chunks_params/readme.md), which each
front a dedicated `_core` engine crate. `chunks_compose`,
`wgsl_try_compose`, and the CLI wiring all live together in this one
`src/lib.rs`.

**Shape:**

```text
compose <names...> [transitive::0|1] [out::<path>]
  -> set_resolve( names, transitive )   // transitive::1 widens to the full dependency closure
  -> set_try_compose( selected )        // topological sort; dependency-before-dependent
  -> composed, dependency-ordered WGSL text
  -> out::<path> given?  compose_write( text, path )  // write to file, print a byte-count summary
     : else              print text to stdout
```

With `transitive::0` (default) the named set must already be
dependency-complete, or composition fails loudly; with `transitive::1`
one root name (e.g. `fbm3`) pulls in its whole chain (`value_noise`,
`hash21`) unasked. Either way the topological sort orders the output
identically, so a closure and the same set spelled out explicitly compose
to the same text. `out::<path>` writes that text to a file instead of
stdout — the write is only attempted after composition already
succeeded, so a failed compose never touches the filesystem.

## Structure

| Path | Responsibility |
|---|---|
| `src/` | `compose` command wiring plus `chunks_compose`/`wgsl_try_compose` — this crate's own core |
| `tests/` | [`shader_chunks_compose_test.rs`](tests/shader_chunks_compose_test.rs) plus the [`docs/cli/`](docs/cli/readme.md) specification mirror at [`tests/docs/cli/`](tests/docs/cli/readme.md) |
| `docs/` | CLI documentation — see [`docs/cli/`](docs/cli/readme.md) for the `compose` command and `plain_text` format |
| `Cargo.toml` | Crate manifest and dependencies |

## Usage

```sh
cargo run -p shader_chunks_compose -- compose hash21 value_noise
cargo run -p shader_chunks_compose -- compose fbm3 transitive::1
cargo run -p shader_chunks_compose -- compose fbm3 transitive::1 out::fbm3_bundle.wgsl
```

```rust
use shader_chunks_compose::{ chunks_compose, compose_write };

let names = vec![ "hash21".to_string(), "value_noise".to_string() ];
let wgsl = chunks_compose( &names, false ).unwrap(); // dependency-ordered regardless of input order

let summary = compose_write( &wgsl, std::path::Path::new( "bundle.wgsl" ) ).unwrap();
assert!( summary.starts_with( "wrote bundle.wgsl" ) );
```

## Errors

[`ComposeCliError`] has three variants: `UnknownChunk` and `Compose`
exit `1` (validation-style, caller-fixable); `Io` exits `2`
(environmental). `UnknownChunk` is a name — or, under `transitive::1`, a
reachable dependency name — not in
[`shader_chunks_core::CHUNKS`]. `Compose` wraps
[`shader_chunks_core::ComposeError`] — a missing dependency or a cyclic
one. `Io` is a failed [`compose_write`] (e.g. a missing parent
directory at `out::`) — only ever reached after composition already
succeeded, so it never masks a validation failure and never leaves a
partial file. [`wgsl_try_compose`] is exposed separately from
[`chunks_compose`] so tests can exercise cyclic/missing-dependency
failures against synthetic fixture WGSL — the real bundled chunk set is
fixed and acyclic, so `CyclicDependency` can never occur through the
name-based [`chunks_compose`] path.

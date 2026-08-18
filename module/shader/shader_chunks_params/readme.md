# shader_chunks_params

**Keywords:** WGSL, Shader Composition, Tunable Parameters, CLI

Parameters utility CLI: the single `tunables` command, rendering
[`shader_chunks_params_core`](../shader_chunks_params_core/readme.md)'s
`//@ param:` discovery as a table. Exposes its command set, help group,
and help examples as data — parameterized by binary name — so the
[`shader_chunks`](../shader_chunks/readme.md) aggregator folds it in
unchanged, while [`run`] serves the same command as the standalone
`shader_chunks_params` binary.

**Shape:**

```text
tunables <name>
  -> shader_chunks_core::chunk_get( name )        // UnknownChunk if not bundled
  -> shader_chunks_params_core::chunk_discover     // //@ param: discovery
  -> plain table: name | kind | type | range | source
```

A chunk with no declared `//@ param:` lines renders an explicit message
("chunk `<name>` declares no tunable parameters") rather than a blank
table or an error — the empty case is a real, intentional answer, not a
failure. `source` distinguishes a declared `range(min, max)` from one
[`shader_chunks_params_core`]'s heuristic inferred.

## Structure

| Path | Responsibility |
|---|---|
| `src/` | `tunables` command wiring over `shader_chunks_params_core`'s `//@ param:` discovery |
| `tests/` | [`tunables_test.rs`](tests/tunables_test.rs) plus the [`docs/cli/`](docs/cli/readme.md) specification mirror at [`tests/docs/cli/`](tests/docs/cli/readme.md) |
| `docs/` | CLI documentation — see [`docs/cli/`](docs/cli/readme.md) for the `tunables` command |
| `Cargo.toml` | Crate manifest and dependencies |

## Usage

```sh
cargo run -p shader_chunks_params -- tunables fbm3
```

```rust
use shader_chunks_params::tunables;

let table = tunables( "fbm3" ).unwrap();
```

[`tunables_of_chunk`] is exposed separately from [`tunables`] so tests
can exercise a chunk descriptor carrying `//@ param:` lines without
depending on any particular bundled chunk's own annotation state — most
bundled chunks declare one or more `//@ param:` lines today, but a
handful of leaf/infrastructure chunks (`hash21`, `hash22`, `palette_cosine`,
`srgb`, `fullscreen_triangle`) still declare none (see
[`shader_chunks_params_core`](../shader_chunks_params_core/readme.md)'s
own readme), so this crate's own tests exercise both the empty-table path
against a real bundled chunk (`hash21`) and the populated-table path
against a self-contained fixture, independent of any bundled chunk's own
annotation state.

## Errors

[`ParamsCliError`] has two variants: `UnknownChunk` (exit `1`,
validation-style) and `Render` (exit `2`, a `data_fmt` table formatter
failure — internal).

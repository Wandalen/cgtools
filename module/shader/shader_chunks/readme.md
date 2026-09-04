# shader_chunks

**Keywords:** WGSL, Shader Composition, CLI, Terminal Tooling, Aggregator

Aggregated terminal CLI for the whole `shader_chunks` utility family —
query, compose, params, preview, and render — under one binary
(`shader_chunks`, short alias `sch`). Read-only/artifact-producing
inspection tool: it never modifies a chunk, only reads the manifest data
[`shader_chunks_core`](../shader_chunks_core/readme.md)'s `CHUNKS`
descriptor table already carries as compile-time fields, composes it,
builds a live preview from it, or renders a static PNG frame of it — no
manifest is parsed at CLI runtime.

**This crate is aggregation only.** `src/lib.rs`'s `run()` concatenates
each utility's command set, help groups, and help examples — query, then
compose, then params, then preview, then render, the order every help
screen and aggregation test pins — and hands the result to
[`shader_chunks_cli_core::run`](../shader_chunks_cli_core/readme.md). All
command logic, argument wiring, and rendering live in the five utility
crates themselves; the two `src/bin/` entry points (`shader_chunks`,
`sch`) are one-line delegates to `shader_chunks::run()`.

## Structure

| Path | Responsibility |
|---|---|
| `src/` | Thin aggregation — concatenates each utility's command set, help groups, and help examples; `src/bin/` delegates |
| `tests/` | [`cli_subprocess_test.rs`](tests/cli_subprocess_test.rs) (aggregation-level) plus the [`docs/cli/`](docs/cli/readme.md) specification mirror at [`tests/docs/cli/`](tests/docs/cli/readme.md) |
| `docs/` | CLI documentation family index — see [`docs/cli/`](docs/cli/readme.md) |
| `Cargo.toml` | Crate manifest and dependencies |

## The utility family

| Crate | Command(s) | Engine crate |
|---|---|---|
| [`shader_chunks_query`](../shader_chunks_query/readme.md) | `list`, `get`, `tags`, `tree` | [`shader_chunks_query_core`](../shader_chunks_query_core/readme.md) |
| [`shader_chunks_compose`](../shader_chunks_compose/readme.md) | `compose` | — (`shader_chunks_core` itself) |
| [`shader_chunks_params`](../shader_chunks_params/readme.md) | `tunables` | [`shader_chunks_params_core`](../shader_chunks_params_core/readme.md) |
| [`shader_chunks_preview`](../shader_chunks_preview/readme.md) | `preview` | [`shader_chunks_preview_core`](../shader_chunks_preview_core/readme.md) |
| [`shader_chunks_render`](../shader_chunks_render/readme.md) | `render` | [`shader_chunks_render_core`](../shader_chunks_render_core/readme.md) |

Every utility also ships its own standalone binary (e.g.
`cargo run -p shader_chunks_query`) with byte-identical behavior for its
own commands — this crate's aggregator binary is a convenience, not the
only way to reach any given command. Dispatch, help rendering, and
exit-code plumbing are shared by every utility (including this
aggregator) through
[`shader_chunks_cli_core`](../shader_chunks_cli_core/readme.md).

## Commands

Commands are partitioned into six groups, documented per leaf crate
([`docs/cli/readme.md`](docs/cli/readme.md)):

| Group | Command | Purpose |
|---|---|---|
| Query | `list [names...]` | Set-shaped queries over the registry; defaults to a table of every chunk |
| Query | `get <names...>` | Same query engine as `list`, defaulting to expanded per-chunk detail records |
| Query | `tags` | Table of every distinct `group:tag` pair and the chunk(s) carrying it |
| Graph | `tree [name]` | Dependency tree for one chunk, or a forest of every chunk nothing depends on |
| Compose | `compose <names...> [transitive::0\|1] [out::<path>]` | Preview WGSL composed from the given chunks, dependency-ordered; `transitive::1` widens to the full dependency closure; `out::<path>` writes it to a file instead of stdout |
| Parameters | `tunables <name>` | One chunk's declared tunable parameters: name, kind, WGSL type, range, range source |
| Preview | `preview [name] [file::<path>] [serve::0\|1]` | Build, naga-validate, and (by default) live-serve a browser preview of one chunk |
| Render | `render [name] [file::<path>] [out::<path>] [size::<n>\|<w>x<h>] [time::<s>] [set::<property>:<value>,...] [all::1]` | Render one headless-GPU frame of a chunk to a static PNG, parameters at defaults unless overridden; `all::1` sweeps every bundled chunk in one pass |
| — | `help` / `<command> help` | Top-level usage / per-command help (`help <command>` works too) |

`list` and `get` are one routine behind two names: both accept the same
21-parameter query surface — positional `names` plus 20 named:
filtering (`pattern::`, `case::`, `tag::`,
`tags_mode::`, `stage::`, `depends_on::`, `transitive::`, `exports::`,
`source::`, `roots::`, `leaves::`), projection (`fields::`, `count::`), and formatting
(`format::`, `sort::`, `order::`, `limit::`, `offset::`, `heading::`,
`width::`) — and differ only in defaults (`list`: all chunks, plain table;
`get`: named chunks required, expanded records). Identical explicit
parameters produce byte-identical output.

## Examples

```sh
cargo run -p shader_chunks -- list
cargo run -p shader_chunks -- list tag::noise format::names
cargo run -p shader_chunks -- list roots::1 fields::name,exports format::markdown
cargo run -p shader_chunks -- list depends_on::hash21 transitive::1 count::1
cargo run -p shader_chunks -- list source::33.33 format::names
cargo run -p shader_chunks -- get hash21
cargo run -p shader_chunks -- get fbm3 format::yaml fields::name,source
cargo run -p shader_chunks -- tags
cargo run -p shader_chunks -- tree fbm3
cargo run -p shader_chunks -- tunables fbm3
cargo run -p shader_chunks -- compose hash21 value_noise
cargo run -p shader_chunks -- compose fbm3 transitive::1
cargo run -p shader_chunks -- compose fbm3 transitive::1 out::fbm3_bundle.wgsl
cargo run -p shader_chunks -- preview fbm3
cargo run -p shader_chunks -- preview fbm3 serve::0
cargo run -p shader_chunks -- render fbm3
cargo run -p shader_chunks -- render fbm3 out::fbm3_far.png size::512 time::2.5
cargo run -p shader_chunks -- render fbm3 set::lacunarity:2.5,gain:0.75
cargo run -p shader_chunks -- render all::1 out::renders/ size::128
```

An unknown chunk name or field, an invalid enum or negative integer value,
an unresolvable `compose` dependency, a chunk that fails naga validation,
a malformed `render` `size::`, or a machine without a usable headless GPU
exits non-zero with a message on stderr — never a panic.
Unmatched open filters (`pattern::`, `tag::`, `stage::`, `exports::`)
yield empty output with exit 0. A pipeline reader hanging up early
(`sch list | head -1`) ends the process quietly with exit 0, per Unix
convention — never a broken-pipe panic.

See [`docs/cli/`](docs/cli/readme.md) for the full
command/parameter/type/format reference and
[`tests/docs/cli/`](tests/docs/cli/readme.md) for the test specifications
mirroring it.

# shader_chunks

**Keywords:** WGSL, Shader Composition, CLI, Terminal Tooling

Terminal CLI for querying, inspecting, and composing the WGSL shader chunks
bundled in [`shader_chunks_core`](../shader_chunks_core/readme.md). Read-only
inspection tool: it never modifies a chunk, only reads and renders the
manifest data `shader_chunks_core`'s `CHUNKS` descriptor table already
carries as compile-time fields — no manifest is parsed at CLI runtime.

Dispatch is real `unilang` (`CommandRegistry` + `Pipeline`, `src/cli.rs`);
tables, records, and the `tree` dependency view are rendered by `data_fmt`;
the help screens — top-level (no arguments, `help`, or `.`) and per-command
(`<command> help`, `help <command>`) — are rendered by `cli_fmt`. Every
command's logic lives in `src/lib.rs` as a plain
`Result<String, CliError>`-returning function, independent of `unilang` —
the `cli` layer only wires argv in and an exit code out, and the two
`src/bin/` entry points (`shader_chunks`, `sch`) are one-line delegates to
it.

## Commands

Commands are partitioned into four groups
([`docs/cli/command_group/`](docs/cli/command_group/readme.md)):

| Group | Command | Purpose |
|---|---|---|
| Query | `list [names...]` | Set-shaped queries over the registry; defaults to a table of every chunk |
| Query | `get <names...>` | Same query engine as `list`, defaulting to expanded per-chunk detail records |
| Query | `tags` | Table of every distinct `group:tag` pair and the chunk(s) carrying it |
| Graph | `tree [name]` | Dependency tree for one chunk, or a forest of every chunk nothing depends on |
| Compose | `compose <names...>` | Preview WGSL composed from the given chunks, dependency-ordered; `transitive::1` widens to the full dependency closure |
| Parameters | `tunables <name>` | One chunk's declared tunable parameters: name, kind, WGSL type, range, range source |
| — | `help` / `<command> help` | Top-level usage / per-command help (`help <command>` works too) |

`list` and `get` are one routine behind two names: both accept the same
20-parameter query surface — positional `names` plus 19 named:
filtering (`pattern::`, `case::`, `tag::`,
`tags_mode::`, `stage::`, `depends_on::`, `transitive::`, `exports::`,
`roots::`, `leaves::`), projection (`fields::`, `count::`), and formatting
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
cargo run -p shader_chunks -- get hash21
cargo run -p shader_chunks -- get fbm3 format::yaml fields::name,source
cargo run -p shader_chunks -- tags
cargo run -p shader_chunks -- tree fbm3
cargo run -p shader_chunks -- tunables fbm3
cargo run -p shader_chunks -- compose hash21 value_noise
cargo run -p shader_chunks -- compose fbm3 transitive::1
```

An unknown chunk name or field, an invalid enum or negative integer value,
or an unresolvable `compose` dependency exits non-zero with a message on
stderr — never a panic. Unmatched open filters (`pattern::`, `tag::`,
`stage::`, `exports::`) yield empty output with exit 0. A pipeline reader
hanging up early (`sch list | head -1`) ends the process quietly with exit
0, per Unix convention — never a broken-pipe panic.

See [`docs/cli/`](docs/cli/readme.md) for the full
command/parameter/type/format reference and
[`tests/docs/cli/`](tests/docs/cli/readme.md) for the test specifications
mirroring it.

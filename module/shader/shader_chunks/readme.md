# shader_chunks

**Keywords:** WGSL, Shader Composition, CLI, Terminal Tooling

Terminal CLI for listing, inspecting, and composing the WGSL shader chunks
bundled in [`shader_chunks_core`](../shader_chunks_core/readme.md). Read-only
inspection tool: it never modifies a chunk, only reads and renders the
manifest data `shader_chunks_core`'s `CHUNKS` descriptor table already
carries as compile-time fields — no manifest is parsed at CLI runtime.

Dispatch is real `unilang` (`CommandRegistry` + `Pipeline`, `src/main.rs`);
`list`/`tags` tables and the `tree` dependency view are rendered by
`data_fmt`; the help screens — top-level (no arguments, `help`, or `.`) and
per-command (`<command> help`, `help <command>`) — are rendered by
`cli_fmt`. Every command's logic lives in `src/lib.rs` as a plain
`Result<String, CliError>`-returning function, independent of `unilang` —
`src/main.rs` only wires argv in and an exit code out.

## Commands

| Command | Purpose |
|---|---|
| `list` | Table of every bundled chunk: name / description / tags / depends_on |
| `get <name>` | Full detail for one chunk: name, description, stage, tags, depends_on, exports |
| `tags` | Table of every distinct `group:tag` pair and the chunk(s) carrying it |
| `tree [name]` | Dependency tree for one chunk, or a forest of every chunk nothing depends on |
| `compose <name...>` | Preview WGSL composed from the given chunks, dependency-ordered |
| `help` / `<command> help` | Top-level usage / per-command help (`help <command>` works too) |

## Examples

```sh
cargo run -p shader_chunks -- list
cargo run -p shader_chunks -- get hash21
cargo run -p shader_chunks -- tags
cargo run -p shader_chunks -- tree fbm3
cargo run -p shader_chunks -- compose hash21 value_noise
```

An unknown chunk name or an unresolvable `compose` dependency exits non-zero
with a message on stderr — never a panic.

See [`docs/cli/`](docs/cli/readme.md) for the full command/parameter/type
reference.

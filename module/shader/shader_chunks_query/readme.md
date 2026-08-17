# shader_chunks_query

**Keywords:** WGSL, Shader Composition, CLI, Terminal Tooling

Query utility CLI: the `list`, `get`, `tags`, and `tree` commands, wiring
[`shader_chunks_query_core`](../shader_chunks_query_core/readme.md)'s
engine into `unilang` `CommandDefinition`s through
[`shader_chunks_cli_core`](../shader_chunks_cli_core/readme.md). Exposes
its command set, help groups, and help examples as data — parameterized
by binary name — so the [`shader_chunks`](../shader_chunks/readme.md)
aggregator folds them in unchanged, while [`run`] serves the same
commands as the standalone `shader_chunks_query` binary. This crate
itself has no `_core` beyond the query engine it wires: it holds no query
logic of its own, only `unilang` argument definitions, per-command
defaults, and error-code mapping.

**Commands:**

| Command | Purpose |
|---|---|
| `list [names...]` | Query chunks: filter, sort, project, format — every chunk by default, plain table |
| `get <names...>` | Same query engine as `list`; names required, expanded per-chunk records by default |
| `tags` | Every distinct `group:tag` pair and the chunk(s) carrying it |
| `tree [name] [reverse::1]` | One chunk's dependency tree, or a forest of every root chunk with no argument; `reverse::1` walks dependents instead |

`list` and `get` are one routine ([`query_routine`], private) behind two
`CommandDefinition`s — [`query_arguments`] builds the identical
19-named-parameter surface for both, differing only in the `defaults`
struct baked into each; a caller passing identical explicit parameters to
both gets byte-identical output.

## Structure

| Path | Responsibility |
|---|---|
| `src/` | `unilang` argument wiring for `list`/`get`/`tags`/`tree` — no query logic of its own |
| `tests/` | No test code (see below) — only the [`docs/cli/`](docs/cli/readme.md) specification mirror at [`tests/docs/cli/`](tests/docs/cli/readme.md) |
| `docs/` | CLI documentation — see [`docs/cli/`](docs/cli/readme.md) for the full command/param/param_group/type/format reference |
| `Cargo.toml` | Crate manifest and dependencies |

## Usage

Deliberately has **no** test code — `tests/` holds only the CLI
documentation mirror (see Structure above). This crate is CLI wiring
only (argument definitions, defaults, error mapping), fully covered by
[`shader_chunks_query_core`](../shader_chunks_query_core/readme.md)'s own
36 direct-call tests (same rendering functions, called without a
`unilang` round-trip) plus [`shader_chunks`](../shader_chunks/readme.md)'s
subprocess suite, which exercises this crate's argument wiring end to
end. This 0/0 count is an intentional architectural asymmetry, not a
coverage gap.

```sh
cargo run -p shader_chunks_query -- list tag::noise format::json
cargo run -p shader_chunks_query -- get hash21
cargo run -p shader_chunks_query -- tags
cargo run -p shader_chunks_query -- tree fbm3
cargo run -p shader_chunks_query -- tree hash21 reverse::1
```

## Error mapping

[`shader_chunks_query_core::QueryError`] is mapped to `unilang`
[`ErrorData`] by a private `query_error` helper: every variant except
`Render` becomes `ErrorCode::ValidationRuleFailed`, `Render` becomes
`ErrorCode::InternalError` — the message and exit code both come straight
from the underlying `QueryError`, so this crate never re-derives error
text of its own.

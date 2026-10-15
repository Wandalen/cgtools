# shader_chunks_validate

**Keywords:** WGSL, Shader Composition, Registry Linting, CLI

Validate utility CLI: the single `validate` command, rendering
[`shader_chunks_validate_core`](../shader_chunks_validate_core/readme.md)'s
registry-wide checks as a human-readable findings report. Exposes its
command set, help group, and help examples as data — parameterized by
binary name — so the [`shader_chunks`](../shader_chunks/readme.md)
aggregator folds it in unchanged, while [`run`] serves the same command
as the standalone `shader_chunks_validate` binary.

**Shape:**

```text
validate
  -> shader_chunks_validate_core::validate( shader_chunks_core::CHUNKS )
  -> zero findings: "registry is clean: <n> chunks, 0 findings"
  -> one or more:    "<n> finding(s):\n\n[chunk] check: message" ( blank-line separated )
```

A clean registry renders an explicit all-clear message rather than blank
output — the empty case is a real, intentional answer, not a failure.
Every finding is reported in one pass; `validate` never stops at the
first problem it finds.

## Structure

| Path | Responsibility |
|---|---|
| `src/` | `validate` command wiring over `shader_chunks_validate_core`'s registry-wide checks |
| `tests/` | [`validate_cli_test.rs`](tests/validate_cli_test.rs) plus the [`docs/cli/`](docs/cli/readme.md) specification mirror at [`tests/docs/cli/`](tests/docs/cli/readme.md) |
| `docs/` | CLI documentation — see [`docs/cli/`](docs/cli/readme.md) for the `validate` command |
| `Cargo.toml` | Crate manifest and dependencies |

## Usage

```sh
cargo run -p shader_chunks_validate -- validate
```

```rust
use shader_chunks_validate::validate;

let report = validate().unwrap();
```

[`validate_chunks`] is exposed separately from [`validate`] so tests can
exercise the rendered report's shape against a local fixture set without
any bundled chunk needing to be broken — the same split
[`shader_chunks_params::tunables_of_chunk`] makes from
[`shader_chunks_params::tunables`], for the same reason.

## Errors

[`ValidateCliError`] has one variant: `FindingsPresent` (exit `1`,
validation-style) — carrying the fully rendered findings report as its
`Display` text. `shader_chunks_validate_core`'s checks are all
non-panicking `Vec`-returning functions over an always-present
compiled-in registry, so there is no second failure mode — no chunk name
to mistype, no render step that can fail.

# shader_chunks_cli_core

**Keywords:** CLI, unilang, Dispatch, Shared Wiring

Shared CLI wiring for the `shader_chunks` utility family. Every utility
binary — [`shader_chunks_query`](../shader_chunks_query/readme.md),
[`shader_chunks_compose`](../shader_chunks_compose/readme.md),
[`shader_chunks_params`](../shader_chunks_params/readme.md),
[`shader_chunks_preview`](../shader_chunks_preview/readme.md) — and the
[`shader_chunks`](../shader_chunks/readme.md) aggregator run through this
one layer instead of each hand-rolling `unilang` registry setup, help
rendering, and exit-code plumbing. It builds the `unilang`
`CommandRegistry` from a supplied [`CommandSet`], dispatches via
`Pipeline`, routes the conventional help spellings (`help`, `.`,
`<command> help`) to `cli_fmt`-rendered help screens, prints `Pipeline`
outputs centrally through pipe-safe write helpers, and maps a
routine-stashed exit code to the actual process exit. Everything is
parameterized by [`CliApp`] — binary name, tagline, help groups/examples,
and commands — so the aggregator and each standalone utility binary get
byte-identical behavior from the same code path.

**Shape:**

```text
CliApp { binary, tagline, groups, examples, commands : CommandSet }
  -> run( app )
       -> registry_build( commands )   // panics on a duplicate command name
       -> Pipeline::process_command_from_argv_simple( argv )
       -> stdout_print / stderr_print  // pipe-safe: EPIPE exits 0, not a panic
       -> process::exit( stashed exit code )
```

A utility's own `commands( binary )` constructor returns a [`CommandSet`]
— one `(CommandDefinition, CommandRoutine)` pair per command; the
aggregator concatenates every utility's set before calling
[`registry_build`]. A routine reports a non-1 exit code by calling
[`error_report`] with the code it wants — `run`'s framework-level
failures (unknown command, missing argument) fall through to the `1`
default untouched.

## Usage

```rust
use shader_chunks_cli_core::{ CliApp, run };

// A standalone utility binary's entire `main`:
fn main()
{
  run( CliApp
  {
    binary : "shader_chunks_query".to_string(),
    tagline : "Query and inspect shader_chunks_core's bundled WGSL chunks.".to_string(),
    groups : shader_chunks_query::help_groups(),
    examples : shader_chunks_query::help_examples( "shader_chunks_query" ),
    commands : shader_chunks_query::commands( "shader_chunks_query" ),
  });
}
```

## Argument-extraction helpers

[`named_arg`] builds one optional `key::value` [`ArgumentDefinition`];
[`arg_string`]/[`arg_bool`]/[`arg_usize`]/[`arg_list`] pull a bound value
back out of a `VerifiedCommand`, each with its own failure behavior —
`arg_usize` is the only one that can fail loudly (a negative value is a
caller mistake, reported via [`error_report`] rather than silently
wrapped). [`names_flatten`] flattens `unilang`'s nested `List`-of-`List`
binding shape for a `multiple: true` positional argument (see its own
doc comment for why the nesting happens) into a plain `Vec<String>` —
every utility with a `names`/`name` positional argument goes through it.

## Pipe safety

`stdout_print`/`stderr_print` exist because the standard `println!`/
`eprintln!` macros panic on any write failure, including `EPIPE` once a
downstream reader like `head` has exited — under `Stdout`'s line
buffering the first post-hangup write fails deterministically. A broken
pipe is not an application error (the reader chose to stop reading), so
[`stdout_print`] exits quietly with `0` on `EPIPE`, per Unix convention,
and only a genuine write failure reports on stderr and exits `2`.

//! Subprocess tests for the `shader_chunks` binary — real process
//! spawns via `assert_cmd`, exercising the actual argv/exit-code path
//! `main.rs` implements (`tests/shader_chunks_test.rs` covers the
//! underlying `src/lib.rs` functions directly, without a subprocess).

use assert_cmd::Command;

fn run_bin( bin : &str, args : &[ &str ] ) -> std::process::Output
{
  Command::cargo_bin( bin )
  .unwrap_or_else( | e | panic!( "{bin} binary should build: {e}" ) )
  .args( args )
  .output()
  .unwrap_or_else( | e | panic!( "{bin} should spawn and run to completion: {e}" ) )
}

fn run( args : &[ &str ] ) -> std::process::Output
{
  run_bin( "shader_chunks", args )
}

fn stdout_of( output : &std::process::Output ) -> String
{
  String::from_utf8_lossy( &output.stdout ).into_owned()
}

#[ test ]
fn list_prints_a_table_with_all_four_bundled_chunks()
{
  let output = run( &[ "list" ] );
  assert!( output.status.success(), "stderr: {}", String::from_utf8_lossy( &output.stderr ) );
  let stdout = stdout_of( &output );
  for name in [ "hash21", "value_noise", "fbm3", "fullscreen_triangle" ]
  {
    assert!( stdout.contains( name ), "list stdout missing `{name}`:\n{stdout}" );
  }
}

#[ test ]
fn get_hash21_prints_full_detail()
{
  let output = run( &[ "get", "hash21" ] );
  assert!( output.status.success(), "stderr: {}", String::from_utf8_lossy( &output.stderr ) );
  let stdout = stdout_of( &output );
  assert!( stdout.contains( "name: hash21" ), "{stdout}" );
  assert!( stdout.contains( "stage: None" ), "{stdout}" );
}

#[ test ]
fn tags_prints_every_distinct_tag()
{
  let output = run( &[ "tags" ] );
  assert!( output.status.success(), "stderr: {}", String::from_utf8_lossy( &output.stderr ) );
  let stdout = stdout_of( &output );
  assert!( stdout.contains( "category:hash" ), "{stdout}" );
  assert!( stdout.contains( "technique:fractal" ), "{stdout}" );
}

#[ test ]
fn tree_fbm3_shows_the_dependency_chain()
{
  let output = run( &[ "tree", "fbm3" ] );
  assert!( output.status.success(), "stderr: {}", String::from_utf8_lossy( &output.stderr ) );
  let stdout = stdout_of( &output );
  let fbm3_pos = stdout.find( "fbm3" ).expect( "fbm3 present" );
  let value_noise_pos = stdout.find( "value_noise" ).expect( "value_noise present" );
  let hash21_pos = stdout.find( "hash21" ).expect( "hash21 present" );
  assert!( fbm3_pos < value_noise_pos && value_noise_pos < hash21_pos, "unexpected tree order:\n{stdout}" );
}

#[ test ]
fn compose_hash21_value_noise_prints_composed_wgsl_in_dependency_order()
{
  let output = run( &[ "compose", "hash21", "value_noise" ] );
  assert!( output.status.success(), "stderr: {}", String::from_utf8_lossy( &output.stderr ) );
  let stdout = stdout_of( &output );
  let hash21_pos = stdout.find( "fn hash21" ).expect( "hash21 present" );
  let value_noise_pos = stdout.find( "fn value_noise" ).expect( "value_noise present" );
  assert!( hash21_pos < value_noise_pos, "hash21 must precede value_noise:\n{stdout}" );
}

#[ test ]
fn get_unknown_chunk_exits_non_zero_without_a_panic_backtrace()
{
  let output = run( &[ "get", "bogus_chunk" ] );
  assert!( !output.status.success(), "expected non-zero exit for an unknown chunk" );
  let stderr = String::from_utf8_lossy( &output.stderr );
  assert!( !stderr.contains( "panicked at" ), "unexpected panic backtrace:\n{stderr}" );
  assert!( stderr.contains( "bogus_chunk" ), "{stderr}" );
}

#[ test ]
fn compose_missing_dependency_exits_non_zero_without_a_panic_backtrace()
{
  let output = run( &[ "compose", "value_noise" ] );
  assert!( !output.status.success(), "expected non-zero exit for a missing dependency" );
  let stderr = String::from_utf8_lossy( &output.stderr );
  assert!( !stderr.contains( "panicked at" ), "unexpected panic backtrace:\n{stderr}" );
}

#[ test ]
fn no_arguments_prints_help_and_exits_zero()
{
  let output = run( &[] );
  assert!( output.status.success(), "stderr: {}", String::from_utf8_lossy( &output.stderr ) );
  let stdout = stdout_of( &output );
  assert!( stdout.contains( "shader_chunks" ), "{stdout}" );
  assert!( stdout.contains( "list" ), "{stdout}" );
}

#[ test ]
fn sch_alias_binary_produces_identical_output_to_shader_chunks()
{
  for args in [ &[ "list" ][ .. ], &[ "get", "hash21" ][ .. ], &[][ .. ] ]
  {
    let primary = run_bin( "shader_chunks", args );
    let alias = run_bin( "sch", args );
    assert_eq!( primary.status.code(), alias.status.code(), "exit code mismatch for {args:?}" );
    assert_eq!( stdout_of( &primary ), stdout_of( &alias ), "stdout mismatch for {args:?}" );
    assert_eq!(
      String::from_utf8_lossy( &primary.stderr ),
      String::from_utf8_lossy( &alias.stderr ),
      "stderr mismatch for {args:?}"
    );
  }
}

// test_kind: bug_reproducer(BUG-103)
/// ## Root Cause
/// `main.rs` printed nothing itself — each command routine `println!`ed its
/// own success content — so every framework-generated help output returned
/// through `result.outputs` (the `.` listing, `.help`, `?`/`??`,
/// `.{command}.help`) was computed and silently dropped, and the
/// conventional spellings had no mapping onto those forms: bare `help`
/// dot-normalized onto the swallowed `.help` builtin (exit 0, zero bytes),
/// while a trailing `help` bound as an ordinary positional argument
/// (`compose help` → "unknown chunk: `help`").
///
/// ## Why Not Caught
/// The only help-path test exercised the bare no-argument invocation, which
/// short-circuits to `print_help()` before the pipeline; every other test
/// ran a real command whose routine printed its own stdout, so the
/// outputs-dropping dispatch path never produced a visible difference.
///
/// ## Fix Applied
/// `main` now prints `result.outputs` after a successful dispatch, routes
/// the top-level spellings (`help`, `.`, `.help`) to `print_help()`, and
/// renders `help <command>` / `<command> help` with `cli_fmt` from the
/// command's registered definition — an unknown target falls through to
/// the `.{target}.help` rewrite, keeping the loud unknown-command failure.
///
/// ## Prevention
/// The help-spelling tests below pin every form (top-level, per-command
/// leading and trailing, no-argument command, unknown command, named-arg
/// escape) plus a prints-exactly-once guard on a normal command.
///
/// ## Pitfall
/// In a routines-print-themselves `unilang` setup, any command the
/// framework answers on its own is invisible until the entry point prints
/// `result.outputs` — and `contains()`-style assertions cannot catch the
/// double-printing that central printing can introduce; count occurrences.
#[ test ]
fn help_word_prints_top_level_usage()
{
  let output = run( &[ "help" ] );
  assert!( output.status.success(), "stderr: {}", String::from_utf8_lossy( &output.stderr ) );
  let stdout = stdout_of( &output );
  assert!( stdout.contains( "Usage: shader_chunks" ), "`help` must print usage, not silence:\n{stdout}" );
  assert!( stdout.contains( "compose" ), "{stdout}" );
}

// test_kind: bug_reproducer(BUG-103)
/// Second symptom of BUG-103 — the trailing-`help` misparse; full
/// Root Cause / Why Not Caught / Fix / Prevention / Pitfall sections are on
/// [`help_word_prints_top_level_usage`].
#[ test ]
fn trailing_help_prints_per_command_help_not_chunk_lookup()
{
  let output = run( &[ "compose", "help" ] );
  let stderr = String::from_utf8_lossy( &output.stderr );
  assert!( output.status.success(), "stderr: {stderr}" );
  assert!( !stderr.contains( "unknown chunk" ), "`compose help` must not be a chunk lookup:\n{stderr}" );
  let stdout = stdout_of( &output );
  assert!( stdout.contains( "Usage: shader_chunks compose <names...>" ), "{stdout}" );
  assert!( stdout.contains( "One or more chunk names" ), "argument hint must render:\n{stdout}" );
  assert!( stdout.contains( "shader_chunks compose hash21 value_noise" ), "example must render:\n{stdout}" );
  assert!( !stdout.contains( "Command: .compose" ), "unilang's generic help format must not leak through:\n{stdout}" );
}

#[ test ]
fn dot_and_dot_help_match_bare_invocation_usage()
{
  let bare = stdout_of( &run( &[] ) );
  for args in [ &[ "." ][ .. ], &[ ".help" ][ .. ], &[ "help", "help" ][ .. ] ]
  {
    let output = run( args );
    assert!( output.status.success(), "stderr for {args:?}: {}", String::from_utf8_lossy( &output.stderr ) );
    assert_eq!( stdout_of( &output ), bare, "{args:?} must print the same usage as the bare invocation" );
  }
}

#[ test ]
fn leading_and_trailing_help_forms_print_identical_per_command_help()
{
  let leading = run( &[ "help", "compose" ] );
  let trailing = run( &[ "compose", "help" ] );
  assert!( leading.status.success(), "stderr: {}", String::from_utf8_lossy( &leading.stderr ) );
  assert_eq!( stdout_of( &leading ), stdout_of( &trailing ), "`help compose` and `compose help` must agree" );
}

#[ test ]
fn no_argument_command_trailing_help_works()
{
  let output = run( &[ "list", "help" ] );
  assert!( output.status.success(), "stderr: {}", String::from_utf8_lossy( &output.stderr ) );
  let stdout = stdout_of( &output );
  assert!( stdout.contains( "Usage: shader_chunks list" ), "{stdout}" );
}

#[ test ]
fn per_command_help_spells_argument_shapes()
{
  // One command per argument shape — required `<name>`, optional `[name]`,
  // repeatable `<name...>` — pinned via the usage line, where the shape
  // derived from each `ArgumentDefinition`'s attributes shows.
  for ( args, expected ) in
  [
    ( &[ "get", "help" ][ .. ], "Usage: shader_chunks get <name>" ),
    ( &[ "tree", "help" ][ .. ], "Usage: shader_chunks tree [name]" ),
    ( &[ "compose", "help" ][ .. ], "Usage: shader_chunks compose <names...>" ),
  ]
  {
    let output = run( args );
    assert!( output.status.success(), "stderr for {args:?}: {}", String::from_utf8_lossy( &output.stderr ) );
    let stdout = stdout_of( &output );
    assert!( stdout.contains( expected ), "{args:?} usage line wrong:\n{stdout}" );
  }
}

#[ test ]
fn unknown_command_help_fails_loudly()
{
  let output = run( &[ "frobnicate", "help" ] );
  assert!( !output.status.success(), "expected non-zero exit for help on an unknown command" );
  let stderr = String::from_utf8_lossy( &output.stderr );
  assert!( !stderr.contains( "panicked at" ), "unexpected panic backtrace:\n{stderr}" );
  assert!( stderr.contains( "not found" ), "{stderr}" );
}

#[ test ]
fn named_argument_help_value_is_not_a_help_request()
{
  // Escape hatch: `name::help` addresses a (hypothetical) chunk literally
  // named `help` — it must reach the chunk lookup, not the help path.
  let output = run( &[ "get", "name::help" ] );
  assert!( !output.status.success(), "expected non-zero exit for an unknown chunk" );
  let stderr = String::from_utf8_lossy( &output.stderr );
  assert!( stderr.contains( "unknown chunk" ), "{stderr}" );
}

#[ test ]
fn command_output_prints_exactly_once()
{
  // Central `result.outputs` printing replaced per-routine `println!`s — a
  // leftover routine print would pass every `contains()` assertion while
  // printing twice; occurrence count is the only guard that catches it.
  let output = run( &[ "get", "hash21" ] );
  assert!( output.status.success(), "stderr: {}", String::from_utf8_lossy( &output.stderr ) );
  let stdout = stdout_of( &output );
  assert_eq!( stdout.matches( "name: hash21" ).count(), 1, "command output must print exactly once:\n{stdout}" );
}

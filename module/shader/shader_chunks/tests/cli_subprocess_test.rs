//! Subprocess tests for the `shader_chunks` binary — real process
//! spawns via `assert_cmd`, exercising the actual argv/exit-code path
//! `src/cli.rs` implements (`tests/shader_chunks_test.rs` covers the
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
fn get_hash21_prints_one_expanded_detail_record()
{
  let output = run( &[ "get", "hash21" ] );
  assert!( output.status.success(), "stderr: {}", String::from_utf8_lossy( &output.stderr ) );
  let stdout = stdout_of( &output );
  assert!( stdout.contains( "-[ RECORD 1 ]" ), "get default must be expanded records:\n{stdout}" );
  assert!( stdout.contains( "| hash21" ), "{stdout}" );
  assert!( stdout.contains( "stage" ), "{stdout}" );
  assert!( stdout.contains( "fn hash21(p: vec2f) -> f32" ), "exports field must render:\n{stdout}" );
  assert!( !stdout.contains( "-[ RECORD 2 ]" ), "one name must yield one record:\n{stdout}" );
}

#[ test ]
fn list_and_get_agree_under_identical_explicit_parameters()
{
  // The unification contract end-to-end: one routine, one engine — with the
  // same explicit parameters the two commands are byte-identical.
  let shared = [ "hash21", "fields::name,stage", "format::expanded" ];
  let list = run( &[ &[ "list" ][ .. ], &shared[ .. ] ].concat() );
  let get = run( &[ &[ "get" ][ .. ], &shared[ .. ] ].concat() );
  assert!( list.status.success(), "stderr: {}", String::from_utf8_lossy( &list.stderr ) );
  assert!( get.status.success(), "stderr: {}", String::from_utf8_lossy( &get.stderr ) );
  assert_eq!( stdout_of( &list ), stdout_of( &get ), "`list` and `get` must share one engine" );
}

#[ test ]
fn list_filters_and_formats_via_named_params()
{
  let output = run( &[ "list", "tag::noise", "format::names" ] );
  assert!( output.status.success(), "stderr: {}", String::from_utf8_lossy( &output.stderr ) );
  assert_eq!( stdout_of( &output ), "value_noise\nfbm3\n" );

  let output = run( &[ "list", "roots::1", "format::names" ] );
  assert!( output.status.success(), "stderr: {}", String::from_utf8_lossy( &output.stderr ) );
  assert_eq!( stdout_of( &output ), "fbm3\nfullscreen_triangle\n" );

  let output = run( &[ "list", "count::1" ] );
  assert!( output.status.success(), "stderr: {}", String::from_utf8_lossy( &output.stderr ) );
  assert_eq!( stdout_of( &output ), "4\n" );
}

#[ test ]
fn invalid_param_values_exit_non_zero_loudly()
{
  for ( args, needle ) in
  [
    ( &[ "list", "format::bogus" ][ .. ], "invalid `format` value" ),
    ( &[ "list", "sort::bogus" ][ .. ], "invalid `sort` value" ),
    ( &[ "list", "order::bogus" ][ .. ], "invalid `order` value" ),
    ( &[ "list", "tags_mode::bogus" ][ .. ], "invalid `tags_mode` value" ),
    ( &[ "list", "limit::-1" ][ .. ], "invalid `limit` value" ),
    ( &[ "list", "fields::bogus" ][ .. ], "unknown field" ),
  ]
  {
    let output = run( args );
    assert!( !output.status.success(), "expected non-zero exit for {args:?}" );
    let stderr = String::from_utf8_lossy( &output.stderr );
    assert!( !stderr.contains( "panicked at" ), "unexpected panic backtrace:\n{stderr}" );
    assert!( stderr.contains( needle ), "stderr for {args:?} must name the offense:\n{stderr}" );
  }
}

#[ test ]
fn get_without_names_fails_loudly_while_list_succeeds()
{
  // Same parameter surface, different defaults: `names` is required for
  // `get`, optional for `list`.
  let get = run( &[ "get" ] );
  assert!( !get.status.success(), "bare `get` must fail" );
  assert!( String::from_utf8_lossy( &get.stderr ).contains( "names" ), "missing-argument error must name `names`" );

  let list = run( &[ "list" ] );
  assert!( list.status.success(), "bare `list` must succeed" );
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
fn tunables_unannotated_real_chunk_prints_explicit_empty_message()
{
  // Task 106's Test Matrix row "`sch tunables <annotated-name>` → expected
  // parameter rows" is not achievable via subprocess without annotating a
  // real bundled chunk with `//@ param:` lines — explicitly out of scope
  // (same Q-03 boundary as task 105). `src/cli.rs`'s `tunables` routine has no
  // branch between the declared-params and zero-params outcomes for a
  // subprocess test to exercise beyond argv wiring, already covered here;
  // `tunables_of_chunk_lists_declared_and_inferred_parameters` in
  // `shader_chunks_test.rs` covers declared-parameter rendering directly
  // against a `LOCAL_GLOW`-style fixture instead.
  let output = run( &[ "tunables", "hash21" ] );
  assert!( output.status.success(), "stderr: {}", String::from_utf8_lossy( &output.stderr ) );
  let stdout = stdout_of( &output );
  assert!( stdout.contains( "hash21" ), "{stdout}" );
  assert!( stdout.contains( "no tunable parameters" ), "empty case must be an explicit message, not blank:\n{stdout}" );
}

#[ test ]
fn tunables_bogus_chunk_exits_non_zero_without_a_panic_backtrace()
{
  let output = run( &[ "tunables", "bogus_chunk" ] );
  assert!( !output.status.success(), "expected non-zero exit for an unknown chunk" );
  let stderr = String::from_utf8_lossy( &output.stderr );
  assert!( !stderr.contains( "panicked at" ), "unexpected panic backtrace:\n{stderr}" );
  assert!( stderr.contains( "unknown chunk" ), "{stderr}" );
}

#[ test ]
fn sch_alias_binary_produces_identical_output_to_shader_chunks()
{
  for args in [ &[ "list" ][ .. ], &[ "get", "hash21" ][ .. ], &[ "tunables", "hash21" ][ .. ], &[][ .. ] ]
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
/// The entry point (then `main.rs`, since dissolved into `src/cli.rs`)
/// printed nothing itself — each command routine `println!`ed its
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
/// short-circuits to `help_print()` before the pipeline; every other test
/// ran a real command whose routine printed its own stdout, so the
/// outputs-dropping dispatch path never produced a visible difference.
///
/// ## Fix Applied
/// `main` now prints `result.outputs` after a successful dispatch, routes
/// the top-level spellings (`help`, `.`, `.help`) to `help_print()`, and
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
  // One command per positional shape — required repeatable `<names...>`,
  // optional repeatable `[names...]`, optional `[name]` — pinned via the
  // usage line, where the shape derived from each `ArgumentDefinition`'s
  // attributes shows; commands with named parameters additionally carry the
  // `[param::value ...]` marker.
  for ( args, expected ) in
  [
    ( &[ "get", "help" ][ .. ], "Usage: shader_chunks get <names...> [param::value ...]" ),
    ( &[ "list", "help" ][ .. ], "Usage: shader_chunks list [names...] [param::value ...]" ),
    ( &[ "tree", "help" ][ .. ], "Usage: shader_chunks tree [name]" ),
    ( &[ "compose", "help" ][ .. ], "Usage: shader_chunks compose <names...> [param::value ...]" ),
  ]
  {
    let output = run( args );
    assert!( output.status.success(), "stderr for {args:?}: {}", String::from_utf8_lossy( &output.stderr ) );
    let stdout = stdout_of( &output );
    assert!( stdout.contains( expected ), "{args:?} usage line wrong:\n{stdout}" );
  }
}

#[ test ]
fn per_command_help_lists_named_params_with_per_command_defaults()
{
  // The two help screens list the identical 20-parameter surface; only the
  // baked-in defaults differ — that difference must be visible to the user.
  let list_help = stdout_of( &run( &[ "list", "help" ] ) );
  let get_help = stdout_of( &run( &[ "help", "get" ] ) );
  for param in
  [
    "pattern::", "case::", "tag::", "tags_mode::", "stage::", "depends_on::", "transitive::",
    "exports::", "roots::", "leaves::", "fields::", "count::", "format::", "sort::", "order::",
    "limit::", "offset::", "heading::", "width::",
  ]
  {
    assert!( list_help.contains( param ), "list help missing `{param}`:\n{list_help}" );
    assert!( get_help.contains( param ), "get help missing `{param}`:\n{get_help}" );
  }
  assert!( list_help.contains( "[default: table]" ), "{list_help}" );
  assert!( list_help.contains( "[default: name,description,tags,depends_on]" ), "{list_help}" );
  assert!( get_help.contains( "[default: expanded]" ), "{get_help}" );
  assert!( get_help.contains( "[default: name,description,stage,tags,depends_on,exports]" ), "{get_help}" );
}

#[ test ]
fn top_level_help_groups_commands_by_responsibility()
{
  // Groups mirror docs/cli/command_group/ — Query, Graph, Compose,
  // Parameters, in that order, with each command under its own group.
  let stdout = stdout_of( &run( &[] ) );
  let query_pos = stdout.find( "Query" ).expect( "Query group present" );
  let graph_pos = stdout.find( "Graph" ).expect( "Graph group present" );
  let compose_pos = stdout.find( "Compose" ).expect( "Compose group present" );
  let parameters_pos = stdout.find( "Parameters" ).expect( "Parameters group present" );
  assert!
  (
    query_pos < graph_pos && graph_pos < compose_pos && compose_pos < parameters_pos,
    "group order wrong:\n{stdout}"
  );
  let list_pos = stdout.find( "list [names...]" ).expect( "list entry present" );
  let tree_pos = stdout.find( "tree [name]" ).expect( "tree entry present" );
  let tunables_pos = stdout.find( "tunables <name>" ).expect( "tunables entry present" );
  assert!( query_pos < list_pos && list_pos < graph_pos, "`list` must sit in the Query group:\n{stdout}" );
  assert!( graph_pos < tree_pos && tree_pos < compose_pos, "`tree` must sit in the Graph group:\n{stdout}" );
  assert!( compose_pos < tunables_pos, "`tunables` must sit in the Parameters group:\n{stdout}" );
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
  // Escape hatch: `names::help` addresses a (hypothetical) chunk literally
  // named `help` — it must reach the chunk lookup, not the help path.
  let output = run( &[ "get", "names::help" ] );
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
  assert_eq!( stdout.matches( "-[ RECORD 1 ]" ).count(), 1, "command output must print exactly once:\n{stdout}" );
  assert_eq!( stdout.matches( "| hash21" ).count(), 1, "command output must print exactly once:\n{stdout}" );
}

// test_kind: bug_reproducer(BUG-108)
/// ## Root Cause
/// Every user-facing write went through `println!`/`eprintln!`, which panic
/// on any write error — including `EPIPE` once the pipe's reader has
/// exited. Rust's `Stdout` is a `LineWriter`, so the first write after the
/// reader closes surfaces the error deterministically: `sch list | true`
/// panicked at "failed printing to stdout: Broken pipe" with exit 101,
/// breaking the crate's documented "never a panic" contract.
///
/// ## Why Not Caught
/// Every subprocess test reads the child's output to completion via
/// `.output()`, so the pipe never closes early; in manual piping, small
/// outputs fit the kernel pipe buffer even when a reader like `head -1`
/// exits first, which made the panic look intermittent instead of certain.
///
/// ## Fix Applied
/// `cli.rs` routes all stdout writes through `stdout_print` — `writeln!` to
/// the locked handle, mapping `BrokenPipe` to a quiet `exit( 0 )` (the Unix
/// convention for a reader that hung up) and any other write error to exit
/// 2 — and all stderr writes through `stderr_print`, which discards write
/// errors so error reporting can never itself become a second failure.
///
/// ## Prevention
/// This test closes the read end of a real OS pipe (`std::io::pipe`)
/// BEFORE spawning the binary, so the child's very first stdout write hits
/// `EPIPE` — no race, no dependence on output size or buffer capacity; the
/// sibling test does the same to stderr and pins the mapped exit code.
///
/// ## Pitfall
/// `println!`/`eprintln!` panic on `EPIPE` by design — a CLI promising
/// "never a panic" must not use them for user-facing output. And a casual
/// `| head -1` smoke check proves nothing: it only breaks the pipe if the
/// reader is already gone by the time the writer writes, so the panic
/// hides until output is large or the scheduler is unlucky.
#[ test ]
fn closed_stdout_pipe_ends_quietly_without_a_panic()
{
  let ( reader, writer ) = std::io::pipe().expect( "OS pipe should be creatable" );
  drop( reader );
  let output = std::process::Command::new( env!( "CARGO_BIN_EXE_shader_chunks" ) )
  .arg( "list" )
  .stdout( writer )
  .output()
  .expect( "shader_chunks should spawn and run to completion" );
  let stderr = String::from_utf8_lossy( &output.stderr );
  assert!( !stderr.contains( "panicked at" ), "closed stdout must not panic:\n{stderr}" );
  assert_eq!( output.status.code(), Some( 0 ), "closed stdout must end quietly with exit 0, got {:?}:\n{stderr}", output.status.code() );
}

// test_kind: bug_reproducer(BUG-108)
/// Second symptom of BUG-108 — `eprintln!` panics the same way when stderr
/// is the closed pipe, turning a mapped exit-1 error report into a 101
/// abort; full Root Cause / Why Not Caught / Fix / Prevention / Pitfall
/// sections are on [`closed_stdout_pipe_ends_quietly_without_a_panic`].
#[ test ]
fn closed_stderr_pipe_still_exits_with_the_mapped_code()
{
  let ( reader, writer ) = std::io::pipe().expect( "OS pipe should be creatable" );
  drop( reader );
  let output = std::process::Command::new( env!( "CARGO_BIN_EXE_shader_chunks" ) )
  .args( [ "get", "bogus_chunk" ] )
  .stderr( writer )
  .output()
  .expect( "shader_chunks should spawn and run to completion" );
  assert_eq!
  (
    output.status.code(),
    Some( 1 ),
    "closed stderr must keep the mapped error exit code, not a panic's 101"
  );
}

#[ test ]
fn compose_single_name_with_transitive_pulls_the_full_dependency_chain()
{
  // Strict remains the default: a bare `compose fbm3` still fails loudly on
  // the missing `value_noise` dependency.
  let strict = run( &[ "compose", "fbm3" ] );
  assert!( !strict.status.success(), "strict compose must still fail on a missing dependency" );

  let output = run( &[ "compose", "fbm3", "transitive::1" ] );
  assert!( output.status.success(), "stderr: {}", String::from_utf8_lossy( &output.stderr ) );
  let stdout = stdout_of( &output );
  let hash21_pos = stdout.find( "fn hash21" ).expect( "hash21 pulled in transitively" );
  let value_noise_pos = stdout.find( "fn value_noise" ).expect( "value_noise pulled in transitively" );
  let fbm3_pos = stdout.find( "fn fbm3" ).expect( "fbm3 present" );
  assert!
  (
    hash21_pos < value_noise_pos && value_noise_pos < fbm3_pos,
    "composed closure must be in dependency order:\n{stdout}"
  );

  // The closure must be exactly what naming the full set explicitly yields.
  let explicit = run( &[ "compose", "hash21", "value_noise", "fbm3" ] );
  assert!( explicit.status.success(), "stderr: {}", String::from_utf8_lossy( &explicit.stderr ) );
  assert_eq!( stdout, stdout_of( &explicit ), "closure must equal the explicit full set" );
}

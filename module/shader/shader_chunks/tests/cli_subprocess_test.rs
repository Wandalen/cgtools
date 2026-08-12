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

//! Direct-call tests for `shader_chunks_compose`'s command logic, plus
//! `out::<path>` file-output tests both in-process and through the real
//! subprocess; see `tests/cli_subprocess_test.rs` in the aggregator for
//! further end-to-end argv and exit-code coverage.

use std::path::PathBuf;
use assert_cmd::Command;
use shader_chunks_compose::{ ComposeCliError, chunks_compose, compose_write, wgsl_try_compose };

fn temp_wgsl( label : &str ) -> PathBuf
{
  std::env::temp_dir().join( format!( "shader_chunks_compose_{label}_{}.wgsl", std::process::id() ) )
}

#[ test ]
fn compose_chunks_orders_hash21_before_value_noise_regardless_of_input_order()
{
  let output = chunks_compose( &[ "value_noise".to_string(), "hash21".to_string() ], false ).expect( "chunks_compose should succeed" );
  let hash21_pos = output.find( "fn hash21" ).expect( "hash21 present" );
  let value_noise_pos = output.find( "fn value_noise" ).expect( "value_noise present" );
  assert!( hash21_pos < value_noise_pos, "hash21 must precede value_noise:\n{output}" );
}

#[ test ]
fn compose_chunks_reports_unknown_chunk_error_for_bogus_name()
{
  let err = chunks_compose( &[ "bogus_chunk".to_string() ], false ).expect_err( "chunks_compose should fail for an unknown name" );
  assert!
  (
    matches!( &err, ComposeCliError::UnknownChunk( name ) if name == "bogus_chunk" ),
    "expected UnknownChunk(\"bogus_chunk\"), got {err:?}"
  );
}

#[ test ]
fn compose_chunks_reports_missing_dependency_error_when_hash21_is_omitted()
{
  let err = chunks_compose( &[ "value_noise".to_string() ], false ).expect_err( "chunks_compose should fail on a missing dependency" );
  assert!
  (
    matches!( &err, ComposeCliError::Compose( shader_chunks_core::ComposeError::MissingDependency { .. } ) ),
    "expected Compose(MissingDependency), got {err:?}"
  );
  assert_eq!( err.exit_code(), 1 );
}

#[ test ]
fn compose_chunks_transitive_closure_equals_the_explicit_full_set()
{
  let closure = chunks_compose( &[ "fbm3".to_string() ], true )
  .expect( "transitive compose of a single root should pull its whole chain" );
  let explicit = chunks_compose
  (
    &[ "hash21".to_string(), "value_noise".to_string(), "fbm3".to_string() ],
    false,
  ).expect( "explicit full set should compose" );
  assert_eq!( closure, explicit, "closure and explicit full set must compose identically" );

  let hash21_pos = closure.find( "fn hash21" ).expect( "hash21 pulled in transitively" );
  let value_noise_pos = closure.find( "fn value_noise" ).expect( "value_noise pulled in transitively" );
  let fbm3_pos = closure.find( "fn fbm3" ).expect( "fbm3 present" );
  assert!
  (
    hash21_pos < value_noise_pos && value_noise_pos < fbm3_pos,
    "closure must compose in dependency order:\n{closure}"
  );
}

#[ test ]
fn compose_chunks_transitive_reports_unknown_chunk_error_for_bogus_name()
{
  // The closure walk resolves every reachable dependency through the same
  // loud lookup as directly-named chunks — a bogus root fails identically
  // under both modes rather than the transitive path masking it.
  let err = chunks_compose( &[ "bogus_chunk".to_string() ], true )
  .expect_err( "transitive compose should fail for an unknown name" );
  assert!
  (
    matches!( &err, ComposeCliError::UnknownChunk( name ) if name == "bogus_chunk" ),
    "expected UnknownChunk(\"bogus_chunk\"), got {err:?}"
  );
}

#[ test ]
fn try_compose_wgsl_reports_cyclic_dependency_error_on_synthetic_fixture()
{
  const A : &str = "//@ name: a\n//@ depends_on: b\nfn a() {}";
  const B : &str = "//@ name: b\n//@ depends_on: a\nfn b() {}";
  let err = wgsl_try_compose( &[ A, B ] ).expect_err( "wgsl_try_compose should fail on a cyclic dependency" );
  assert!
  (
    matches!( &err, ComposeCliError::Compose( shader_chunks_core::ComposeError::CyclicDependency( _ ) ) ),
    "expected Compose(CyclicDependency), got {err:?}"
  );
}

#[ test ]
fn compose_write_writes_the_composed_text_and_returns_a_byte_count_summary()
{
  let content = chunks_compose( &[ "hash21".to_string() ], false ).expect( "chunks_compose should succeed" );
  let out = temp_wgsl( "write_happy" );
  let summary = compose_write( &content, &out ).expect( "compose_write should succeed" );
  assert!( summary.contains( &format!( "wrote {}", out.display() ) ), "summary: {summary}" );
  assert!( summary.contains( &format!( "({} bytes wgsl)", content.len() ) ), "summary: {summary}" );
  let written = std::fs::read_to_string( &out ).expect( "the written file must be readable" );
  assert_eq!( written, content );
  let _ = std::fs::remove_file( &out );
}

#[ test ]
fn compose_write_to_an_unwritable_path_is_an_io_error_with_exit_code_2()
{
  let out = std::env::temp_dir()
  .join( format!( "shader_chunks_compose_no_such_dir_{}", std::process::id() ) )
  .join( "bundle.wgsl" );
  let err = compose_write( "fn hash21(p: vec2f) -> f32 { return 0.0; }", &out )
  .expect_err( "a missing parent directory must fail the write" );
  assert!( matches!( err, ComposeCliError::Io( _ ) ), "expected Io, got {err:?}" );
  assert_eq!( err.exit_code(), 2 );
  assert!( err.to_string().contains( "io error" ), "got: {err}" );
  assert!( !out.exists() );
}

#[ test ]
fn subprocess_compose_writes_the_file_and_prints_the_summary()
{
  let out = temp_wgsl( "subprocess_happy" );
  let output = Command::cargo_bin( "shader_chunks_compose" ).expect( "binary builds" )
  .args( [ "compose", "hash21", "value_noise", &format!( "out::{}", out.display() ) ] )
  .output()
  .expect( "runs" );
  assert!( output.status.success(), "stderr: {}", String::from_utf8_lossy( &output.stderr ) );
  let stdout = String::from_utf8_lossy( &output.stdout );
  assert!( stdout.contains( "wrote" ), "stdout: {stdout}" );
  assert!( stdout.contains( "bytes wgsl" ), "stdout: {stdout}" );
  assert!( !stdout.contains( "fn hash21" ), "the composed text itself should go to the file, not stdout: {stdout}" );

  let written = std::fs::read_to_string( &out ).expect( "the written file must be readable" );
  let direct = chunks_compose( &[ "hash21".to_string(), "value_noise".to_string() ], false ).expect( "chunks_compose should succeed" );
  assert_eq!( written, direct );
  let _ = std::fs::remove_file( &out );
}

#[ test ]
fn subprocess_compose_without_out_prints_composed_text_to_stdout()
{
  let output = Command::cargo_bin( "shader_chunks_compose" ).expect( "binary builds" )
  .args( [ "compose", "hash21", "value_noise" ] )
  .output()
  .expect( "runs" );
  assert!( output.status.success(), "stderr: {}", String::from_utf8_lossy( &output.stderr ) );
  let stdout = String::from_utf8_lossy( &output.stdout );
  assert!( stdout.contains( "fn hash21" ), "omitting `out::` must keep printing composed text to stdout: {stdout}" );
  assert!( !stdout.contains( "wrote" ), "no file-write summary should appear without `out::`: {stdout}" );
}

#[ test ]
fn subprocess_compose_with_out_and_transitive_writes_the_full_closure()
{
  let out = temp_wgsl( "subprocess_transitive" );
  let output = Command::cargo_bin( "shader_chunks_compose" ).expect( "binary builds" )
  .args( [ "compose", "fbm3", "transitive::1", &format!( "out::{}", out.display() ) ] )
  .output()
  .expect( "runs" );
  assert!( output.status.success(), "stderr: {}", String::from_utf8_lossy( &output.stderr ) );

  let written = std::fs::read_to_string( &out ).expect( "the written file must be readable" );
  let direct = chunks_compose( &[ "fbm3".to_string() ], true ).expect( "transitive chunks_compose should succeed" );
  assert_eq!( written, direct );
  let hash21_pos = written.find( "fn hash21" ).expect( "hash21 pulled in transitively" );
  let value_noise_pos = written.find( "fn value_noise" ).expect( "value_noise pulled in transitively" );
  let fbm3_pos = written.find( "fn fbm3" ).expect( "fbm3 present" );
  assert!( hash21_pos < value_noise_pos && value_noise_pos < fbm3_pos, "closure must compose in dependency order:\n{written}" );
  let _ = std::fs::remove_file( &out );
}

#[ test ]
fn subprocess_compose_out_to_unwritable_path_fails_with_exit_2()
{
  let out = std::env::temp_dir()
  .join( format!( "shader_chunks_compose_subprocess_no_such_dir_{}", std::process::id() ) )
  .join( "bundle.wgsl" );
  let output = Command::cargo_bin( "shader_chunks_compose" ).expect( "binary builds" )
  .args( [ "compose", "hash21", &format!( "out::{}", out.display() ) ] )
  .output()
  .expect( "runs" );
  assert_eq!( output.status.code(), Some( 2 ) );
  let stderr = String::from_utf8_lossy( &output.stderr );
  assert!( stderr.contains( "io error" ), "stderr: {stderr}" );
  assert!( !out.exists() );
}

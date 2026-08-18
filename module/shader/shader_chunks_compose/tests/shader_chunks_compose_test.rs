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

/// ## Root Cause
/// `unilang`'s semantic analyzer collects a named argument into a
/// `Value::List` whenever the same key is supplied more than once on argv
/// -- unconditionally, regardless of whether the argument's own
/// `ArgumentDefinition` declared `multiple: true`
/// ( `unilang::semantic::argument_binding::bind_argument_values`, guarded
/// by `if parser_args.len() > 1` before ever consulting
/// `arg_def.attributes.multiple` ). `shader_chunks_cli_core::arg_string`
/// only matched `Value::String`/`Value::Enum`; any other variant --
/// including this `Value::List` -- fell through its catch-all `_ => None`
/// arm, making a duplicated `out::` indistinguishable from `out::` never
/// having been supplied at all.
///
/// ## Why Not Caught
/// Every existing `out::` test ( `subprocess_compose_writes_the_file_and_prints_the_summary`
/// et al. ) supplies the key exactly once; nothing in this crate's suite
/// exercised a repeated named key, so the silent fallback to `arg_string`'s
/// `None` branch -- which happens to be `compose`'s own well-tested "no
/// `out::` given" behavior -- was never distinguished from the "given but
/// ignored" case.
///
/// ## Fix Applied
/// Added `shader_chunks_cli_core::arg_string_checked` / `arg_bool_checked`
/// -- new, additive helpers ( not a breaking change to the existing
/// `arg_string`/`arg_bool` still used by sibling utility crates ) that
/// explicitly match `Value::List` and return a loud `ErrorData`/exit-1
/// naming the duplicated key instead of silently falling through.
/// `shader_chunks_compose`'s `cmd_compose` routine now uses the checked
/// variants for its own `out`/`transitive` parameters.
///
/// ## Prevention
/// Any future single-value named-argument extractor added to
/// `shader_chunks_cli_core` must explicitly handle `Value::List` -- never
/// rely on a catch-all `_` arm for a `Value` match, since `unilang` can
/// produce a list for *any* named key the moment it is repeated, whether
/// or not the argument was declared `multiple`.
///
/// ## Pitfall
/// A `Value` catch-all arm silently conflates two very different
/// situations -- "the key was never supplied" and "the key was supplied,
/// just not in the shape this extractor expects" -- collapsing both into
/// the same fallback. When that fallback is itself a valid, well-tested
/// behavior in its own right ( here: "no `out::`, print to stdout" ), the
/// silent misrouting is invisible until someone diffs the *destination* of
/// the output against what was actually requested.
#[ test ]
fn subprocess_compose_out_given_twice_fails_loudly_instead_of_silently_printing_to_stdout()
{
  let out_a = temp_wgsl( "dup_out_a" );
  let out_b = temp_wgsl( "dup_out_b" );
  let output = Command::cargo_bin( "shader_chunks_compose" ).expect( "binary builds" )
  .args( [ "compose", "hash21", &format!( "out::{}", out_a.display() ), &format!( "out::{}", out_b.display() ) ] )
  .output()
  .expect( "runs" );

  assert!( !output.status.success(), "a duplicated `out::` must not silently succeed" );
  assert_eq!( output.status.code(), Some( 1 ), "duplicated named argument is a caller-fixable validation error, exit 1" );
  let stderr = String::from_utf8_lossy( &output.stderr );
  assert!( stderr.contains( "out" ), "stderr should name the offending parameter: {stderr}" );
  let stdout = String::from_utf8_lossy( &output.stdout );
  assert!( !stdout.contains( "fn hash21" ), "must not silently fall back to printing the composed WGSL to stdout: {stdout}" );
  assert!( !out_a.exists(), "neither out:: value must receive a partial/stale write" );
  assert!( !out_b.exists(), "neither out:: value must receive a partial/stale write" );
}

/// ## Root Cause
/// Same underlying defect as
/// `subprocess_compose_out_given_twice_fails_loudly_instead_of_silently_printing_to_stdout`
/// ( see its doc comment for the full mechanism ), manifesting through
/// `arg_bool` instead of `arg_string`: a duplicated `transitive::1
/// transitive::1` binds as `Value::List([Boolean(true), Boolean(true)])`,
/// which `arg_bool`'s `Some(Value::Boolean(flag)) => *flag, _ => default`
/// match silently resolved to `default` ( `false` ) via its catch-all arm.
///
/// ## Why Not Caught
/// Every existing `transitive` test supplies the key at most once; no
/// existing case repeats a named boolean flag.
///
/// ## Fix Applied
/// `cmd_compose` now calls the new `arg_bool_checked`, which matches
/// `Value::List` explicitly and returns a loud exit-1 error instead of
/// silently taking `default`.
///
/// ## Prevention
/// See the sibling `out::` test's doc comment -- same rule: never let a
/// `Value` match's catch-all arm absorb `Value::List`.
///
/// ## Pitfall
/// Silently falling back to `transitive`'s default of `false` doesn't just
/// drop a flag -- it routes execution into a *different, unrelated* error
/// path ( `MissingDependency`, since the closure was never widened ),
/// which reads like a legitimate dependency-resolution failure rather than
/// a swallowed duplicate-argument bug, making misdiagnosis likely.
#[ test ]
fn subprocess_compose_transitive_given_twice_fails_loudly_instead_of_silently_defaulting_to_false()
{
  let output = Command::cargo_bin( "shader_chunks_compose" ).expect( "binary builds" )
  .args( [ "compose", "fbm3", "transitive::1", "transitive::1" ] )
  .output()
  .expect( "runs" );

  assert!( !output.status.success(), "a duplicated `transitive::` must not silently succeed" );
  assert_eq!( output.status.code(), Some( 1 ), "duplicated named argument is a caller-fixable validation error, exit 1" );
  let stderr = String::from_utf8_lossy( &output.stderr );
  assert!( stderr.contains( "transitive" ), "stderr should name the offending parameter: {stderr}" );
  assert!( !stderr.contains( "was not passed to compose" ), "must not silently fall into the unrelated MissingDependency path: {stderr}" );
}

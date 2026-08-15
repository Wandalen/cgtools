//! Tests for the `preview` command's build-and-write pipeline ( `serve::0`
//! keeps the browser out of the loop ) and its loud rejection paths, both
//! in-process and through the real subprocess.

use std::path::PathBuf;
use assert_cmd::Command;
use shader_chunks_preview::{ bundle_prepare, preview, web_crate_dir, PreviewCliError, PreviewTarget };

fn temp_wgsl( label : &str ) -> PathBuf
{
  std::env::temp_dir().join( format!( "shader_chunks_preview_{label}_{}.wgsl", std::process::id() ) )
}

#[ test ]
fn name_target_prepares_a_validated_bundle()
{
  let bundle = bundle_prepare( &PreviewTarget::Name( "fbm3".to_string() ) ).expect( "fbm3 is bundled and previewable" );
  assert_eq!( bundle.target, "fbm3" );
  assert!( !bundle.parameters.is_empty(), "a value chunk gets the synthesized preview_scale slider" );
}

#[ test ]
fn unknown_name_is_rejected_with_the_shared_unknown_chunk_text()
{
  let err = bundle_prepare( &PreviewTarget::Name( "bogus_chunk".to_string() ) ).expect_err( "should fail" );
  assert!( matches!( &err, PreviewCliError::UnknownChunk( name ) if name == "bogus_chunk" ), "expected UnknownChunk, got {err:?}" );
  assert_eq!( err.exit_code(), 1 );
  assert_eq!( err.to_string(), "unknown chunk: `bogus_chunk` (see `list` for valid names)" );
}

#[ test ]
fn missing_file_is_an_io_error_with_exit_code_2()
{
  let err = bundle_prepare( &PreviewTarget::File( "no/such/file.wgsl".to_string() ) ).expect_err( "should fail" );
  assert!( matches!( err, PreviewCliError::Io( _ ) ), "expected Io, got {err:?}" );
  assert_eq!( err.exit_code(), 2 );
}

#[ test ]
fn file_target_prepares_the_same_bundle_as_the_bundled_name()
{
  // The file mode's contract: a local file's text goes through the exact
  // path a bundled chunk's `wgsl` field does — proven by feeding a real
  // bundled chunk's own text back through `file::`.
  let source = temp_wgsl( "file_target" );
  let chunk = shader_chunks_core::chunk_get( "fbm3" ).expect( "fbm3 is bundled" );
  std::fs::write( &source, chunk.wgsl ).expect( "temp chunk file writes" );

  let from_file = bundle_prepare( &PreviewTarget::File( source.display().to_string() ) )
  .expect( "a bundled chunk's own text must prepare via file::" );
  let from_name = bundle_prepare( &PreviewTarget::Name( "fbm3".to_string() ) ).expect( "fbm3 prepares" );
  assert_eq!( from_file.wgsl, from_name.wgsl, "file mode and name mode must build the identical bundle" );
  assert!( !from_file.parameters.is_empty(), "the synthesized preview_scale slider must survive file mode" );
  let _ = std::fs::remove_file( &source );
}

#[ test ]
fn preview_without_serve_writes_the_bundle_into_the_web_runner_crate()
{
  let summary = preview( &PreviewTarget::Name( "fbm3".to_string() ), false ).expect( "fbm3 previews" );
  assert!( summary.contains( "-preview.json" ), "summary must name the written file: {summary}" );
  assert!( summary.contains( "target: fbm3" ), "summary must name the target: {summary}" );

  let written = web_crate_dir().join( "-preview.json" );
  let json = std::fs::read_to_string( &written ).expect( "-preview.json must exist in the web runner crate" );
  let bundle : shader_chunks_preview_core::PreviewBundle = serde_json::from_str( &json ).expect( "written bundle must round-trip" );
  assert_eq!( bundle.target, "fbm3" );
}

#[ test ]
fn subprocess_preview_serve_0_succeeds_and_prints_the_summary()
{
  let output = Command::cargo_bin( "shader_chunks_preview" ).expect( "binary builds" )
  .args( [ "preview", "fbm3", "serve::0" ] )
  .output()
  .expect( "runs" );
  assert!( output.status.success(), "stderr: {}", String::from_utf8_lossy( &output.stderr ) );
  let stdout = String::from_utf8_lossy( &output.stdout );
  assert!( stdout.contains( "naga-validated" ), "stdout: {stdout}" );
  assert!( stdout.contains( "preview_scale" ), "stdout: {stdout}" );
}

#[ test ]
fn subprocess_preview_with_unknown_name_fails_with_exit_1()
{
  let output = Command::cargo_bin( "shader_chunks_preview" ).expect( "binary builds" )
  .args( [ "preview", "bogus_chunk", "serve::0" ] )
  .output()
  .expect( "runs" );
  assert_eq!( output.status.code(), Some( 1 ) );
  let stderr = String::from_utf8_lossy( &output.stderr );
  assert!( stderr.contains( "unknown chunk: `bogus_chunk`" ), "stderr: {stderr}" );
}

#[ test ]
fn subprocess_preview_with_no_target_fails_loudly()
{
  let output = Command::cargo_bin( "shader_chunks_preview" ).expect( "binary builds" )
  .args( [ "preview", "serve::0" ] )
  .output()
  .expect( "runs" );
  assert_eq!( output.status.code(), Some( 1 ) );
  let stderr = String::from_utf8_lossy( &output.stderr );
  assert!( stderr.contains( "exactly one target" ), "stderr: {stderr}" );
}

#[ test ]
fn subprocess_preview_with_both_targets_fails_loudly()
{
  let output = Command::cargo_bin( "shader_chunks_preview" ).expect( "binary builds" )
  .args( [ "preview", "fbm3", "file::whatever.wgsl", "serve::0" ] )
  .output()
  .expect( "runs" );
  assert_eq!( output.status.code(), Some( 1 ) );
  let stderr = String::from_utf8_lossy( &output.stderr );
  assert!( stderr.contains( "exactly one target" ), "stderr: {stderr}" );
}

#[ test ]
fn subprocess_preview_with_bad_serve_value_is_rejected_by_coercion()
{
  // The timeout is the safety net: if coercion ever silently accepted a
  // non-boolean, the default-on serve would block on the browser server —
  // a timeout kill yields code None, which fails the matches! below loudly.
  let output = Command::cargo_bin( "shader_chunks_preview" ).expect( "binary builds" )
  .args( [ "preview", "fbm3", "serve::maybe" ] )
  .timeout( std::time::Duration::from_secs( 30 ) )
  .output()
  .expect( "runs" );
  assert!( matches!( output.status.code(), Some( code ) if code != 0 ), "a non-boolean serve:: must be rejected: {:?}", output.status );
}

#[ test ]
fn subprocess_help_lists_the_preview_group()
{
  let output = Command::cargo_bin( "shader_chunks_preview" ).expect( "binary builds" )
  .args( [ "help" ] )
  .output()
  .expect( "runs" );
  assert!( output.status.success() );
  let stdout = String::from_utf8_lossy( &output.stdout );
  assert!( stdout.contains( "Preview" ), "stdout: {stdout}" );
  assert!( stdout.contains( "preview [name]" ), "stdout: {stdout}" );
}

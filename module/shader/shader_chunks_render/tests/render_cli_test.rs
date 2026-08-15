//! Tests for the `render` command's build-render-write pipeline and its
//! loud rejection paths, both in-process and through the real subprocess.
//! The happy paths render on the real headless GPU and re-open the
//! written PNG to prove the file is a valid image of the requested size.

use std::path::PathBuf;
use assert_cmd::Command;
use shader_chunks_preview::PreviewTarget;
use shader_chunks_render::{ RenderCliError, out_path_of, render_to_png, size_parse };

fn temp_png( label : &str ) -> PathBuf
{
  std::env::temp_dir().join( format!( "shader_chunks_render_{label}_{}.png", std::process::id() ) )
}

fn temp_wgsl( label : &str ) -> PathBuf
{
  std::env::temp_dir().join( format!( "shader_chunks_render_{label}_{}.wgsl", std::process::id() ) )
}

#[ test ]
fn size_parse_accepts_square_and_explicit_forms()
{
  assert_eq!( size_parse( "256" ).unwrap(), ( 256, 256 ) );
  assert_eq!( size_parse( "1" ).unwrap(), ( 1, 1 ) );
  assert_eq!( size_parse( "128x64" ).unwrap(), ( 128, 64 ) );
  assert_eq!( size_parse( " 32 x 16 " ).unwrap(), ( 32, 16 ) );
}

#[ test ]
fn size_parse_rejects_zero_missing_and_junk_sides()
{
  for raw in [ "0", "0x5", "5x0", "", "x", "64x", "x64", "abc", "-1", "1x2x3", "256X256", "1.5" ]
  {
    let err = size_parse( raw ).expect_err( "must reject" );
    assert!( matches!( &err, RenderCliError::InvalidSize( bad ) if bad == raw ), "raw `{raw}` gave {err:?}" );
    assert_eq!( err.exit_code(), 1 );
    assert!( err.to_string().contains( &format!( "`{raw}`" ) ), "message must quote the offending value: {err}" );
  }
}

#[ test ]
fn out_path_default_derives_from_the_target()
{
  let named = PreviewTarget::Name( "fbm3".to_string() );
  assert_eq!( out_path_of( &named, None ), PathBuf::from( "fbm3.png" ) );
  let file = PreviewTarget::File( "some/dir/-harness.wgsl".to_string() );
  assert_eq!( out_path_of( &file, None ), PathBuf::from( "-harness.png" ) );
  assert_eq!( out_path_of( &named, Some( "custom/spot.png".to_string() ) ), PathBuf::from( "custom/spot.png" ) );
}

#[ test ]
fn name_target_renders_a_png_of_the_requested_size()
{
  let out = temp_png( "name_target" );
  let summary = render_to_png( &PreviewTarget::Name( "fbm3".to_string() ), ( 24, 24 ), 0.0, &out )
  .expect( "fbm3 is bundled, previewable, and renderable" );
  assert!( summary.contains( "target: fbm3" ), "summary must name the target: {summary}" );
  assert!( summary.contains( "naga-validated" ), "summary must state validation ran: {summary}" );
  assert!( summary.contains( "preview_scale = 8" ), "summary must list the baked parameter values: {summary}" );

  let image = image::open( &out ).expect( "the written PNG must be a readable image" );
  assert_eq!( ( image.width(), image.height() ), ( 24, 24 ) );
  let _ = std::fs::remove_file( &out );
}

#[ test ]
fn unknown_name_is_rejected_with_the_shared_unknown_chunk_text()
{
  let out = temp_png( "unknown_name" );
  let err = render_to_png( &PreviewTarget::Name( "bogus_chunk".to_string() ), ( 8, 8 ), 0.0, &out )
  .expect_err( "should fail" );
  assert_eq!( err.exit_code(), 1 );
  assert_eq!( err.to_string(), "unknown chunk: `bogus_chunk` (see `list` for valid names)" );
  assert!( !out.exists(), "no PNG may be written on failure" );
}

#[ test ]
fn unpreviewable_chunk_is_rejected_before_any_gpu_work()
{
  // fullscreen_triangle is a vertex-stage struct+entry-point pair, not a
  // fragment chunk or a `fn(p: vec2f) -> T` value function — the one
  // chunk in the registry with no previewable export of any shape — so
  // bundle building rejects it and the render layer never touches the GPU.
  let out = temp_png( "unpreviewable" );
  let err = render_to_png( &PreviewTarget::Name( "fullscreen_triangle".to_string() ), ( 8, 8 ), 0.0, &out )
  .expect_err( "should fail" );
  assert_eq!( err.exit_code(), 1 );
  assert!( err.to_string().contains( "chunk `fullscreen_triangle` is not previewable" ), "got: {err}" );
  assert!( !out.exists(), "no PNG may be written on failure" );
}

#[ test ]
fn missing_file_is_an_io_error_with_exit_code_2()
{
  let out = temp_png( "missing_file" );
  let err = render_to_png( &PreviewTarget::File( "no/such/file.wgsl".to_string() ), ( 8, 8 ), 0.0, &out )
  .expect_err( "should fail" );
  assert_eq!( err.exit_code(), 2 );
  assert!( !out.exists(), "no PNG may be written on failure" );
}

#[ test ]
fn file_target_renders_the_same_chunk_text_as_a_bundled_name()
{
  // The file mode's contract: a local file's text goes through the exact
  // path a bundled chunk's `wgsl` field does — proven by feeding a real
  // bundled chunk's own text back through `file::`.
  let source = temp_wgsl( "file_target" );
  let chunk = shader_chunks_core::chunk_get( "fbm3" ).expect( "fbm3 is bundled" );
  std::fs::write( &source, chunk.wgsl ).expect( "temp chunk file writes" );

  let out = temp_png( "file_target" );
  let summary = render_to_png( &PreviewTarget::File( source.display().to_string() ), ( 16, 16 ), 0.0, &out )
  .expect( "a bundled chunk's own text must render via file::" );
  assert!( summary.contains( "naga-validated" ), "summary must state validation ran: {summary}" );

  let image = image::open( &out ).expect( "the written PNG must be a readable image" );
  assert_eq!( ( image.width(), image.height() ), ( 16, 16 ) );
  let _ = std::fs::remove_file( &source );
  let _ = std::fs::remove_file( &out );
}

#[ test ]
fn unwritable_out_path_is_an_io_error_with_exit_code_2()
{
  let out = std::env::temp_dir()
  .join( format!( "shader_chunks_render_no_such_dir_{}", std::process::id() ) )
  .join( "frame.png" );
  let err = render_to_png( &PreviewTarget::Name( "fbm3".to_string() ), ( 8, 8 ), 0.0, &out )
  .expect_err( "a missing parent directory must fail the write" );
  assert!( matches!( err, RenderCliError::Io( _ ) ), "expected Io, got {err:?}" );
  assert_eq!( err.exit_code(), 2 );
  assert!( err.to_string().contains( "io error" ), "got: {err}" );
  assert!( !out.exists() );
}

#[ test ]
fn subprocess_render_writes_the_png_and_prints_the_summary()
{
  let out = temp_png( "subprocess_happy" );
  let output = Command::cargo_bin( "shader_chunks_render" ).expect( "binary builds" )
  .args( [ "render", "fbm3", &format!( "out::{}", out.display() ), "size::16" ] )
  .output()
  .expect( "runs" );
  assert!( output.status.success(), "stderr: {}", String::from_utf8_lossy( &output.stderr ) );
  let stdout = String::from_utf8_lossy( &output.stdout );
  assert!( stdout.contains( "naga-validated" ), "stdout: {stdout}" );
  assert!( stdout.contains( "16x16 px" ), "stdout: {stdout}" );

  let image = image::open( &out ).expect( "the written PNG must be a readable image" );
  assert_eq!( ( image.width(), image.height() ), ( 16, 16 ) );
  let _ = std::fs::remove_file( &out );
}

#[ test ]
fn subprocess_render_with_unknown_name_fails_with_exit_1()
{
  let output = Command::cargo_bin( "shader_chunks_render" ).expect( "binary builds" )
  .args( [ "render", "bogus_chunk" ] )
  .output()
  .expect( "runs" );
  assert_eq!( output.status.code(), Some( 1 ) );
  let stderr = String::from_utf8_lossy( &output.stderr );
  assert!( stderr.contains( "unknown chunk: `bogus_chunk`" ), "stderr: {stderr}" );
}

#[ test ]
fn subprocess_render_with_no_target_fails_loudly()
{
  let output = Command::cargo_bin( "shader_chunks_render" ).expect( "binary builds" )
  .args( [ "render" ] )
  .output()
  .expect( "runs" );
  assert_eq!( output.status.code(), Some( 1 ) );
  let stderr = String::from_utf8_lossy( &output.stderr );
  assert!( stderr.contains( "exactly one target" ), "stderr: {stderr}" );
}

#[ test ]
fn subprocess_render_with_both_targets_fails_loudly()
{
  let output = Command::cargo_bin( "shader_chunks_render" ).expect( "binary builds" )
  .args( [ "render", "fbm3", "file::whatever.wgsl" ] )
  .output()
  .expect( "runs" );
  assert_eq!( output.status.code(), Some( 1 ) );
  let stderr = String::from_utf8_lossy( &output.stderr );
  assert!( stderr.contains( "exactly one target" ), "stderr: {stderr}" );
}

#[ test ]
fn subprocess_render_with_bad_size_fails_with_exit_1()
{
  let output = Command::cargo_bin( "shader_chunks_render" ).expect( "binary builds" )
  .args( [ "render", "fbm3", "size::0" ] )
  .output()
  .expect( "runs" );
  assert_eq!( output.status.code(), Some( 1 ) );
  let stderr = String::from_utf8_lossy( &output.stderr );
  assert!( stderr.contains( "invalid `size` value: `0`" ), "stderr: {stderr}" );
}

#[ test ]
fn subprocess_render_with_non_numeric_time_is_rejected_by_coercion()
{
  let output = Command::cargo_bin( "shader_chunks_render" ).expect( "binary builds" )
  .args( [ "render", "fbm3", "time::later" ] )
  .output()
  .expect( "runs" );
  assert_ne!( output.status.code(), Some( 0 ), "a non-numeric time must not render" );
}

#[ test ]
fn subprocess_render_with_fractional_time_succeeds()
{
  let out = temp_png( "fractional_time" );
  let output = Command::cargo_bin( "shader_chunks_render" ).expect( "binary builds" )
  .args( [ "render", "fbm3", &format!( "out::{}", out.display() ), "size::8", "time::2.5" ] )
  .output()
  .expect( "runs" );
  assert!( output.status.success(), "stderr: {}", String::from_utf8_lossy( &output.stderr ) );
  let stdout = String::from_utf8_lossy( &output.stdout );
  assert!( stdout.contains( "time: 2.5" ), "summary must echo the frozen instant: {stdout}" );
  assert!( out.exists(), "the PNG must be written" );
  let _ = std::fs::remove_file( &out );
}

#[ test ]
fn subprocess_render_with_integer_time_token_succeeds()
{
  let out = temp_png( "integer_time" );
  let output = Command::cargo_bin( "shader_chunks_render" ).expect( "binary builds" )
  .args( [ "render", "fbm3", &format!( "out::{}", out.display() ), "size::8", "time::2" ] )
  .output()
  .expect( "runs" );
  assert!( output.status.success(), "stderr: {}", String::from_utf8_lossy( &output.stderr ) );
  let stdout = String::from_utf8_lossy( &output.stdout );
  assert!( stdout.contains( "time: 2" ), "summary must echo the frozen instant: {stdout}" );
  assert!( out.exists(), "the PNG must be written" );
  let _ = std::fs::remove_file( &out );
}

#[ test ]
fn subprocess_render_with_non_finite_time_is_rejected()
{
  // Whichever layer catches it — unilang's float coercion or the
  // routine's own finiteness guard — a non-finite time must exit
  // non-zero and leave no file behind.
  let out = temp_png( "non_finite_time" );
  let output = Command::cargo_bin( "shader_chunks_render" ).expect( "binary builds" )
  .args( [ "render", "fbm3", &format!( "out::{}", out.display() ), "time::inf" ] )
  .output()
  .expect( "runs" );
  assert!( matches!( output.status.code(), Some( code ) if code != 0 ), "a non-finite time must not render: {:?}", output.status );
  assert!( !out.exists(), "no PNG may be written on failure" );
}

#[ test ]
fn subprocess_help_lists_the_render_group()
{
  let output = Command::cargo_bin( "shader_chunks_render" ).expect( "binary builds" )
  .args( [ "help" ] )
  .output()
  .expect( "runs" );
  assert!( output.status.success() );
  let stdout = String::from_utf8_lossy( &output.stdout );
  assert!( stdout.contains( "Render" ), "stdout: {stdout}" );
  assert!( stdout.contains( "render [name]" ), "stdout: {stdout}" );
}

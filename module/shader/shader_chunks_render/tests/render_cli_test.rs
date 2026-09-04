//! Tests for the `render` command's build-render-write pipeline and its
//! loud rejection paths, both in-process and through the real subprocess.
//! The happy paths render on the real headless GPU and re-open the
//! written PNG to prove the file is a valid image of the requested size.
//! The `render_all_to_png_*`/`subprocess_render_all_*` tests cover the
//! `all::1` batch mode against the real bundled chunk registry, including
//! its one known-unpreviewable chunk (`fullscreen_triangle`, skipped, not
//! failed).

use std::path::PathBuf;
use assert_cmd::Command;
use shader_chunks_preview::PreviewTarget;
use shader_chunks_render::{ BatchOutcome, RenderCliError, batch_summary, out_path_of, overrides_apply, overrides_parse, render_all_to_png, render_to_png, size_parse };

fn temp_png( label : &str ) -> PathBuf
{
  std::env::temp_dir().join( format!( "shader_chunks_render_{label}_{}.png", std::process::id() ) )
}

fn temp_wgsl( label : &str ) -> PathBuf
{
  std::env::temp_dir().join( format!( "shader_chunks_render_{label}_{}.wgsl", std::process::id() ) )
}

fn temp_dir( label : &str ) -> PathBuf
{
  let dir = std::env::temp_dir().join( format!( "shader_chunks_render_batch_{label}_{}", std::process::id() ) );
  let _ = std::fs::remove_dir_all( &dir );
  dir
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
  let summary = render_to_png( &PreviewTarget::Name( "fbm3".to_string() ), ( 24, 24 ), 0.0, &[], &out )
  .expect( "fbm3 is bundled, previewable, and renderable" );
  assert!( summary.contains( "target: fbm3" ), "summary must name the target: {summary}" );
  assert!( summary.contains( "naga-validated" ), "summary must state validation ran: {summary}" );
  assert!( summary.contains( "preview_scale = 8" ), "summary must list the baked parameter values: {summary}" );

  let image = image::open( &out ).expect( "the written PNG must be a readable image" );
  assert_eq!( ( image.width(), image.height() ), ( 24, 24 ) );
  let _ = std::fs::remove_file( &out );
}

// test_kind: bug_reproducer(BUG-286)
/// ## Root Cause
/// `palette_cosine_preview` threaded its documented-fixed "canonical
/// rainbow parameterization" ( `readme.md`'s own Visualization section:
/// `a = b = vec3f(0.5)`, `c = vec3f(1.0)`, `d = vec3f(0.0, 0.33, 0.67)` )
/// through as six independent tunable arguments instead of hardcoding
/// them. `shader_chunks_params_core`'s range inference always defaults an
/// argument to its own declared range's midpoint, independently of every
/// other argument -- so `phase_r`/`phase_g`/`phase_b` ( each declared
/// `range(0.0, 1.0)` ) all defaulted to the identical `0.5`, collapsing
/// the three RGB channels ( which only differ from each other via the
/// phase vector `d` ) to the same value at every pixel: the default
/// render was flat grayscale, not the documented rainbow.
/// ## Why Not Caught
/// The existing structural test ( `shader_chunks_preview_core`'s
/// `preview_bundle_test.rs::vec3_value_chunk_gets_a_synthesized_harness` )
/// only asserted the generated WGSL called `palette_cosine_preview` with
/// the right variable names -- it never rendered a frame or inspected a
/// pixel, so a semantically-monochrome-by-default demo passed it cleanly.
/// ## Fix Applied
/// `palette_cosine_preview` ( `shader/palette_cosine/palette_cosine.wgsl` )
/// now takes only `p: vec2f` and hardcodes the readme's own canonical
/// values directly, matching every sibling bespoke-demo chunk's
/// convention of baking fixed compositional constants into the wrapper
/// body rather than exposing them as sliders.
/// ## Prevention
/// This test renders the chunk for real on the headless GPU and decodes
/// the PNG, so a future regression that re-collapses the channels (e.g.
/// re-exposing the phase spread as independently-defaulted sliders)
/// fails on an actual pixel-color assertion, not just a string match.
/// ## Pitfall
/// Any other chunk that threads N structurally-identical, identically
/// -ranged parameters through where the demo's meaning depends on them
/// differing from each other ( not merely each being independently
/// "reasonable" ) is vulnerable to the same collapse; the rest of the
/// bundled set was audited for this specific shape during BUG-286's
/// investigation and no other instance was found.
#[ test ]
fn palette_cosine_default_render_shows_distinct_channels_not_flat_grayscale()
{
  let out = temp_png( "palette_cosine_color" );
  render_to_png( &PreviewTarget::Name( "palette_cosine".to_string() ), ( 64, 64 ), 0.0, &[], &out )
  .expect( "palette_cosine is bundled, previewable, and renderable" );

  let image = image::open( &out ).expect( "the written PNG must be a readable image" ).to_rgb8();
  let max_channel_spread = image.pixels()
  .map( | pixel | { let [ r, g, b ] = pixel.0; r.max( g ).max( b ) - r.min( g ).min( b ) } )
  .max()
  .expect( "a 64x64 image has pixels" );
  assert!
  (
    max_channel_spread > 40,
    "the canonical rainbow parameterization must produce visibly distinct R/G/B channels somewhere in the frame, \
    not a flat grayscale image (max channel spread across all pixels was only {max_channel_spread})"
  );
  let _ = std::fs::remove_file( &out );
}

#[ test ]
fn set_override_replaces_the_named_parameters_default_value()
{
  // fbm3's own defaults are the range midpoints -- lacunarity 2.0, gain
  // 0.5 -- so 2.5/0.75 can only appear in the summary via the override.
  let out = temp_png( "set_override" );
  let overrides = overrides_parse( &[ "lacunarity:2.5".to_string(), "gain:0.75".to_string() ] )
  .expect( "well-formed override tokens must parse" );
  let summary = render_to_png( &PreviewTarget::Name( "fbm3".to_string() ), ( 8, 8 ), 0.0, &overrides, &out )
  .expect( "a valid override must still render" );
  assert!( summary.contains( "lacunarity = 2.5" ), "summary must show the overridden value, not the 2.0 default: {summary}" );
  assert!( summary.contains( "gain = 0.75" ), "summary must show the overridden value, not the 0.5 default: {summary}" );
  let _ = std::fs::remove_file( &out );
}

#[ test ]
fn set_override_rejects_an_unknown_parameter_name()
{
  let out = temp_png( "set_unknown" );
  let overrides = overrides_parse( &[ "bogus_param:1.0".to_string() ] ).expect( "well-formed token parses" );
  let err = render_to_png( &PreviewTarget::Name( "fbm3".to_string() ), ( 8, 8 ), 0.0, &overrides, &out )
  .expect_err( "an unknown property must be rejected" );
  assert_eq!( err.exit_code(), 1 );
  assert!( matches!( &err, RenderCliError::UnknownOverrideParameter { name, .. } if name == "bogus_param" ), "got: {err:?}" );
  let message = err.to_string();
  assert!( message.contains( "lacunarity" ) && message.contains( "gain" ) && message.contains( "preview_scale" ),
    "message must list every valid property: {message}" );
  assert!( !out.exists(), "no PNG may be written on failure" );
}

#[ test ]
fn overrides_parse_rejects_a_token_missing_its_separator()
{
  let err = overrides_parse( &[ "lacunarity_no_colon".to_string() ] ).expect_err( "must reject" );
  assert!( matches!( &err, RenderCliError::InvalidOverride( raw ) if raw == "lacunarity_no_colon" ), "got: {err:?}" );
  assert_eq!( err.exit_code(), 1 );
}

#[ test ]
fn overrides_parse_rejects_a_non_finite_or_non_numeric_value()
{
  for raw in [ "lacunarity:inf", "lacunarity:-inf", "lacunarity:nan", "lacunarity:abc", "lacunarity:" ]
  {
    let err = overrides_parse( &[ raw.to_string() ] ).expect_err( "must reject" );
    assert!( matches!( &err, RenderCliError::InvalidOverride( bad ) if bad == raw ), "raw `{raw}` gave {err:?}" );
  }
}

#[ test ]
fn overrides_apply_lets_a_later_override_of_the_same_property_win()
{
  let mut bundle = shader_chunks_preview::bundle_prepare( &PreviewTarget::Name( "fbm3".to_string() ) )
  .expect( "fbm3 is bundled and previewable" );
  let overrides = overrides_parse( &[ "gain:0.1".to_string(), "gain:0.9".to_string() ] )
  .expect( "well-formed tokens parse" );
  overrides_apply( &mut bundle, &overrides ).expect( "both overrides name a real property" );
  let gain = bundle.parameters.iter().find( | p | p.property == "gain" ).expect( "gain exists on fbm3" );
  assert!( ( gain.value - 0.9 ).abs() < f64::EPSILON, "the later override must win: got {}", gain.value );
}

// test_kind: bug_reproducer(BUG-155)
/// ## Root Cause
/// This test's expected string dropped the `shader_chunks ` prefix that
/// `shader_chunks_preview::PreviewCliError::UnknownChunk` actually produces
/// (`shader_chunks_preview/src/lib.rs`) -- the exact error `render_to_png`
/// returns verbatim via `bundle_prepare`, since neither the `render` nor
/// `preview` standalone binary has a local `list` command of its own to
/// point users at (only the `shader_chunks`/`sch` aggregator and the
/// `shader_chunks_query` binary do).
/// ## Why Not Caught
/// No compile-time link exists between this test's hardcoded string and
/// the sibling `shader_chunks_preview::tests::unknown_name_is_rejected_with_the_shared_unknown_chunk_text`
/// test that already asserts the correct (prefixed) text for the same
/// shared `PreviewCliError::UnknownChunk` value -- the two tests' names
/// promise identical expectations but nothing enforced it.
/// ## Fix Applied
/// Added the missing `shader_chunks ` prefix so this assertion matches
/// `shader_chunks_preview`'s own passing assertion for the same error.
/// ## Prevention
/// None added -- fixing the existing assertion to match the one shared
/// error text is the whole fix; no new test needed since a test already
/// exists on both sides of the "shared" text once corrected.
/// ## Pitfall
/// A test name that says "shared" with a sibling test is only a comment,
/// not a guarantee -- two independently hardcoded string literals can
/// still silently drift out of sync when one crate's doc/CLI convention
/// changes and the other's copy isn't updated alongside it.
#[ test ]
fn unknown_name_is_rejected_with_the_shared_unknown_chunk_text()
{
  let out = temp_png( "unknown_name" );
  let err = render_to_png( &PreviewTarget::Name( "bogus_chunk".to_string() ), ( 8, 8 ), 0.0, &[], &out )
  .expect_err( "should fail" );
  assert_eq!( err.exit_code(), 1 );
  assert_eq!( err.to_string(), "unknown chunk: `bogus_chunk` (see `shader_chunks list` for valid names)" );
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
  let err = render_to_png( &PreviewTarget::Name( "fullscreen_triangle".to_string() ), ( 8, 8 ), 0.0, &[], &out )
  .expect_err( "should fail" );
  assert_eq!( err.exit_code(), 1 );
  assert!( err.to_string().contains( "chunk `fullscreen_triangle` is not previewable" ), "got: {err}" );
  assert!( !out.exists(), "no PNG may be written on failure" );
}

#[ test ]
fn missing_file_is_an_io_error_with_exit_code_2()
{
  let out = temp_png( "missing_file" );
  let err = render_to_png( &PreviewTarget::File( "no/such/file.wgsl".to_string() ), ( 8, 8 ), 0.0, &[], &out )
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
  let summary = render_to_png( &PreviewTarget::File( source.display().to_string() ), ( 16, 16 ), 0.0, &[], &out )
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
  let err = render_to_png( &PreviewTarget::Name( "fbm3".to_string() ), ( 8, 8 ), 0.0, &[], &out )
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
fn subprocess_render_with_set_override_shows_the_overridden_value()
{
  let out = temp_png( "subprocess_set" );
  let output = Command::cargo_bin( "shader_chunks_render" ).expect( "binary builds" )
  .args( [ "render", "fbm3", &format!( "out::{}", out.display() ), "size::8", "set::lacunarity:2.5,gain:0.75" ] )
  .output()
  .expect( "runs" );
  assert!( output.status.success(), "stderr: {}", String::from_utf8_lossy( &output.stderr ) );
  let stdout = String::from_utf8_lossy( &output.stdout );
  assert!( stdout.contains( "lacunarity = 2.5" ), "stdout: {stdout}" );
  assert!( stdout.contains( "gain = 0.75" ), "stdout: {stdout}" );
  let _ = std::fs::remove_file( &out );
}

#[ test ]
fn subprocess_render_with_unknown_set_parameter_fails_with_exit_1()
{
  let output = Command::cargo_bin( "shader_chunks_render" ).expect( "binary builds" )
  .args( [ "render", "fbm3", "set::bogus_param:1.0" ] )
  .output()
  .expect( "runs" );
  assert_eq!( output.status.code(), Some( 1 ) );
  let stderr = String::from_utf8_lossy( &output.stderr );
  assert!( stderr.contains( "unknown parameter: `bogus_param`" ), "stderr: {stderr}" );
}

// test_kind: bug_reproducer(BUG-285)
/// ## Root Cause
/// The `render` routine read `name`/`file`/`size`/`out` through
/// `shader_chunks_cli_core::arg_string`, whose catch-all `_ => None` arm
/// cannot distinguish "argument absent" from "argument supplied more than
/// once" -- `unilang` binds a repeated named key to `Value::List`
/// regardless of the argument's own declared `multiple` attribute, so
/// `out::a.png out::b.png` fell through the same arm as "absent" and
/// silently rendered to the default `<target>.png` path instead of
/// erroring, discarding one of the two paths the caller explicitly typed.
/// ## Why Not Caught
/// No existing test in this file passed a named `key::value` argument
/// twice in one invocation -- every prior test supplied each parameter at
/// most once.
/// ## Fix Applied
/// The routine now reads `name`/`file`/`size`/`out`/`all` through
/// `arg_string_checked`/`arg_bool_checked` (`shader_chunks_render/src/lib.rs`),
/// which returns a loud `ValidationRuleFailed` naming the key and its
/// repeat count instead of silently falling back to the default.
/// ## Prevention
/// This subprocess test locks in the loud-failure behavior for `out`;
/// sibling crates `shader_chunks_query`/`shader_chunks_preview` gained
/// their own matching regression tests in the same fix (BUG-285).
/// ## Pitfall
/// `arg_usize` (unused directly in this crate's own call sites, but shared
/// via `shader_chunks_cli_core`) has the identical catch-all shape and
/// remains unfixed elsewhere -- a known, not a forgotten, gap.
#[ test ]
fn subprocess_render_with_duplicated_out_fails_loudly_instead_of_using_the_default_path()
{
  let output = Command::cargo_bin( "shader_chunks_render" ).expect( "binary builds" )
  .args( [ "render", "fbm3", "out::a.png", "out::b.png" ] )
  .output()
  .expect( "runs" );
  assert_eq!( output.status.code(), Some( 1 ), "stdout: {}", String::from_utf8_lossy( &output.stdout ) );
  let stderr = String::from_utf8_lossy( &output.stderr );
  assert!( stderr.contains( "`out` was given 2 times" ), "stderr: {stderr}" );
  assert!( !std::path::Path::new( "a.png" ).exists() && !std::path::Path::new( "b.png" ).exists(), "no PNG may be written on failure" );
}

// test_kind: bug_reproducer(BUG-419)
/// ## Root Cause
/// `arg_time` read the `time::` argument through a bare `Value` match
/// whose catch-all `_ => 0.0` arm cannot distinguish "argument absent"
/// from "argument supplied twice" -- `unilang` binds a repeated named key
/// to `Value::List` regardless of the argument's own declared `multiple`
/// attribute, so `time::1.0 time::2.0` fell through the same arm as
/// "absent" and silently rendered at `time = 0.0` instead of erroring.
/// ## Why Not Caught
/// No existing test in this file passed `time::` twice in one invocation
/// -- every prior `time::`-related test supplied it at most once
/// (`subprocess_render_with_fractional_time_succeeds`,
/// `subprocess_render_with_integer_time_token_succeeds`,
/// `subprocess_render_with_non_finite_time_is_rejected`,
/// `subprocess_render_with_non_numeric_time_is_rejected_by_coercion`).
/// ## Fix Applied
/// `arg_time` (`shader_chunks_render/src/lib.rs`) now has an explicit
/// `Value::List` arm returning a loud `ValidationRuleFailed` naming the
/// key and its repeat count, matching the same-class fix BUG-285 already
/// applied to this routine's other arguments (`name`/`file`/`size`/`out`/
/// `all`).
/// ## Prevention
/// This subprocess test locks in the loud-failure behavior for a
/// duplicated `time::`.
/// ## Pitfall
/// `arg_time` was added to this crate after BUG-285's own fix pass, so it
/// never inherited that fix's `arg_string_checked`/`arg_bool_checked`
/// migration -- a defect class fixed once in a shared helper family does
/// not retroactively protect a new, independently-written local helper
/// using the same bare-`_`-catch-all shape.
#[ test ]
fn subprocess_render_with_duplicated_time_fails_loudly_instead_of_defaulting_to_zero()
{
  let out = temp_png( "duplicated_time" );
  let output = Command::cargo_bin( "shader_chunks_render" ).expect( "binary builds" )
  .args( [ "render", "fbm3", &format!( "out::{}", out.display() ), "time::1.0", "time::2.0" ] )
  .output()
  .expect( "runs" );
  assert_eq!( output.status.code(), Some( 1 ), "stdout: {}", String::from_utf8_lossy( &output.stdout ) );
  let stderr = String::from_utf8_lossy( &output.stderr );
  assert!( stderr.contains( "`time` was given 2 times" ), "stderr: {stderr}" );
  assert!( !out.exists(), "no PNG may be written on failure" );
}

#[ test ]
fn subprocess_render_with_malformed_set_token_fails_with_exit_1()
{
  let output = Command::cargo_bin( "shader_chunks_render" ).expect( "binary builds" )
  .args( [ "render", "fbm3", "set::no_colon_here" ] )
  .output()
  .expect( "runs" );
  assert_eq!( output.status.code(), Some( 1 ) );
  let stderr = String::from_utf8_lossy( &output.stderr );
  assert!( stderr.contains( "invalid `set` override: `no_colon_here`" ), "stderr: {stderr}" );
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

#[ test ]
fn render_all_to_png_creates_the_out_dir_and_covers_every_bundled_chunk_with_no_failures()
{
  let dir = temp_dir( "coverage" );
  assert!( !dir.exists(), "the dir must not pre-exist, to prove auto-creation" );
  let outcomes = render_all_to_png( ( 8, 8 ), 0.0, &dir ).expect( "the fresh temp dir must be creatable" );
  assert!( dir.is_dir(), "render_all_to_png must create out_dir" );
  assert_eq!( outcomes.len(), shader_chunks_core::CHUNKS.len(), "one outcome per bundled chunk" );
  let failed : Vec< _ > = outcomes.iter().filter_map( | o | match o
  {
    BatchOutcome::Failed { name, error } => Some( format!( "{name}: {error}" ) ),
    _ => None,
  }).collect();
  assert!( failed.is_empty(), "no bundled chunk should fail to render: {failed:?}" );
  let _ = std::fs::remove_dir_all( &dir );
}

#[ test ]
fn render_all_to_png_writes_a_valid_png_for_every_rendered_chunk()
{
  let dir = temp_dir( "writes" );
  let outcomes = render_all_to_png( ( 8, 8 ), 0.0, &dir ).expect( "the fresh temp dir must be creatable" );
  let rendered : Vec< _ > = outcomes.iter().filter_map( | o | match o
  {
    BatchOutcome::Rendered { name, path } => Some( ( name.clone(), path.clone() ) ),
    _ => None,
  }).collect();
  assert!( !rendered.is_empty(), "at least one bundled chunk must render" );
  for ( name, path ) in &rendered
  {
    let image = image::open( path ).unwrap_or_else( | err | panic!( "{name}'s PNG at {} must be readable: {err}", path.display() ) );
    assert_eq!( ( image.width(), image.height() ), ( 8, 8 ), "{name}'s PNG must be the requested size" );
  }
  let _ = std::fs::remove_dir_all( &dir );
}

#[ test ]
fn render_all_to_png_skips_the_known_unpreviewable_chunk_without_writing_a_file()
{
  let dir = temp_dir( "skip" );
  let outcomes = render_all_to_png( ( 8, 8 ), 0.0, &dir ).expect( "the fresh temp dir must be creatable" );
  let skipped = outcomes.iter().find( | o | matches!( o, BatchOutcome::Skipped { name, .. } if name == "fullscreen_triangle" ) )
  .unwrap_or_else( || panic!( "fullscreen_triangle must be Skipped, got: {outcomes:?}" ) );
  if let BatchOutcome::Skipped { reason, .. } = skipped
  {
    assert!( !reason.is_empty(), "the skip reason must not be empty" );
  }
  assert!( !dir.join( "fullscreen_triangle.png" ).exists(), "a skipped chunk must not write a file" );
  let _ = std::fs::remove_dir_all( &dir );
}

#[ test ]
fn batch_summary_lists_each_outcome_and_a_totals_line()
{
  let outcomes = vec!
  [
    BatchOutcome::Rendered { name : "a".to_string(), path : PathBuf::from( "out/a.png" ) },
    BatchOutcome::Skipped { name : "b".to_string(), reason : "not previewable".to_string() },
    BatchOutcome::Failed { name : "c".to_string(), error : RenderCliError::InvalidSize( "bad".to_string() ) },
  ];
  let text = batch_summary( &outcomes );
  assert!( text.contains( "a: wrote out/a.png" ), "got: {text}" );
  assert!( text.contains( "b: skipped (not previewable)" ), "got: {text}" );
  assert!( text.contains( "c: failed (invalid `size` value: `bad`" ), "got: {text}" );
  assert!( text.contains( "3 chunks: 1 rendered, 1 skipped, 1 failed" ), "got: {text}" );
}

#[ test ]
fn subprocess_render_all_writes_a_png_per_chunk_into_a_freshly_created_dir_and_reports_totals()
{
  let dir = temp_dir( "subprocess_all" );
  let output = Command::cargo_bin( "shader_chunks_render" ).expect( "binary builds" )
  .args( [ "render", "all::1", &format!( "out::{}", dir.display() ), "size::8" ] )
  .output()
  .expect( "runs" );
  assert!( output.status.success(), "stderr: {}", String::from_utf8_lossy( &output.stderr ) );
  let stdout = String::from_utf8_lossy( &output.stdout );
  assert!( stdout.contains( "chunks:" ), "stdout must print the totals line: {stdout}" );
  assert!( stdout.contains( "0 failed" ), "no bundled chunk should fail: {stdout}" );
  assert!( dir.join( "fbm3.png" ).exists(), "fbm3.png must be written into the freshly created out dir" );
  let _ = std::fs::remove_dir_all( &dir );
}

#[ test ]
fn subprocess_render_all_rejects_a_name_target()
{
  let output = Command::cargo_bin( "shader_chunks_render" ).expect( "binary builds" )
  .args( [ "render", "fbm3", "all::1" ] )
  .output()
  .expect( "runs" );
  assert_eq!( output.status.code(), Some( 1 ) );
  let stderr = String::from_utf8_lossy( &output.stderr );
  assert!( stderr.contains( "cannot be combined" ), "stderr: {stderr}" );
}

#[ test ]
fn subprocess_render_all_rejects_a_file_target()
{
  let output = Command::cargo_bin( "shader_chunks_render" ).expect( "binary builds" )
  .args( [ "render", "file::whatever.wgsl", "all::1" ] )
  .output()
  .expect( "runs" );
  assert_eq!( output.status.code(), Some( 1 ) );
  let stderr = String::from_utf8_lossy( &output.stderr );
  assert!( stderr.contains( "cannot be combined" ), "stderr: {stderr}" );
}

#[ test ]
fn subprocess_render_all_rejects_set_overrides()
{
  let output = Command::cargo_bin( "shader_chunks_render" ).expect( "binary builds" )
  .args( [ "render", "all::1", "set::lacunarity:2.5" ] )
  .output()
  .expect( "runs" );
  assert_eq!( output.status.code(), Some( 1 ) );
  let stderr = String::from_utf8_lossy( &output.stderr );
  assert!( stderr.contains( "cannot be combined" ), "stderr: {stderr}" );
}

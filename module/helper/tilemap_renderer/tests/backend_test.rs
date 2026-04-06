//! Backend trait contract tests.
//!
//! A minimal `TestBackend` implementation verifies that:
//! - the `Backend` trait is implementable,
//! - `RenderError` variants format correctly,
//! - `Capabilities::default()` returns the zero state.

mod helpers;
use helpers::empty_assets;

use tilemap_renderer::assets::Assets;
use tilemap_renderer::backend::{ Backend, Capabilities, Output, RenderError };
use tilemap_renderer::commands::RenderCommand;

// ============================================================================
// Minimal test backend
// ============================================================================

struct TestBackend
{
  assets_loaded : bool,
  last_command_count : usize,
  width : u32,
  height : u32,
}

impl TestBackend
{
  fn new() -> Self
  {
    Self { assets_loaded : false, last_command_count : 0, width : 0, height : 0 }
  }
}

impl Backend for TestBackend
{
  fn load_assets( &mut self, _assets : &Assets ) -> Result< (), RenderError >
  {
    self.assets_loaded = true;
    Ok( () )
  }

  fn submit( &mut self, commands : &[ RenderCommand ] ) -> Result< (), RenderError >
  {
    self.last_command_count = commands.len();
    Ok( () )
  }

  fn output( &self ) -> Result< Output, RenderError >
  {
    Ok( Output::String( "test".to_string() ) )
  }

  fn resize( &mut self, width : u32, height : u32 )
  {
    self.width = width;
    self.height = height;
  }

  fn capabilities( &self ) -> Capabilities
  {
    Capabilities::default()
  }
}

// ============================================================================
// Tests
// ============================================================================

#[ test ]
fn load_assets_valid()
{
  let mut b = TestBackend::new();
  let assets = empty_assets();
  assert!( b.load_assets( &assets ).is_ok() );
  assert!( b.assets_loaded );
}

#[ test ]
fn load_assets_empty()
{
  let mut b = TestBackend::new();
  assert!( b.load_assets( &empty_assets() ).is_ok() );
}

#[ test ]
fn submit_empty_slice()
{
  let mut b = TestBackend::new();
  assert!( b.submit( &[] ).is_ok() );
  assert_eq!( b.last_command_count, 0 );
}

#[ test ]
fn submit_clear_command()
{
  use tilemap_renderer::commands::Clear;
  let mut b = TestBackend::new();
  let cmds = [ RenderCommand::Clear( Clear { color : [ 0.0, 0.0, 0.0, 1.0 ] } ) ];
  assert!( b.submit( &cmds ).is_ok() );
  assert_eq!( b.last_command_count, 1 );
}

#[ test ]
fn output_returns_string()
{
  let b = TestBackend::new();
  match b.output().unwrap()
  {
    Output::String( s ) => assert_eq!( s, "test" ),
    other => panic!( "unexpected output: {other:?}" ),
  }
}

#[ test ]
fn render_error_missing_asset_display()
{
  let e = RenderError::MissingAsset( 42 );
  let s = format!( "{e}" );
  assert!( s.contains( "missing asset" ), "got: {s}" );
  assert!( s.contains( "42" ), "got: {s}" );
}

#[ test ]
fn render_error_unsupported_display()
{
  let e = RenderError::Unsupported( "gradients" );
  let s = format!( "{e}" );
  assert!( s.contains( "unsupported" ), "got: {s}" );
  assert!( s.contains( "gradients" ), "got: {s}" );
}

#[ test ]
fn render_error_backend_error_display()
{
  let e = RenderError::BackendError( "gpu lost".to_string() );
  let s = format!( "{e}" );
  assert!( s.contains( "backend error" ), "got: {s}" );
  assert!( s.contains( "gpu lost" ), "got: {s}" );
}

#[ test ]
fn resize_stores_dimensions()
{
  let mut b = TestBackend::new();
  b.resize( 800, 600 );
  assert_eq!( b.width, 800 );
  assert_eq!( b.height, 600 );
}

#[ test ]
fn capabilities_default_all_false()
{
  let c = Capabilities::default();
  assert!( !c.paths );
  assert!( !c.text );
  assert!( !c.meshes );
  assert!( !c.sprites );
  assert!( !c.batches );
  assert!( !c.gradients );
  assert!( !c.patterns );
  assert!( !c.clip_masks );
  assert!( !c.effects );
  assert!( !c.blend_modes );
  assert!( !c.text_on_path );
  assert_eq!( c.max_texture_size, 0 );
}

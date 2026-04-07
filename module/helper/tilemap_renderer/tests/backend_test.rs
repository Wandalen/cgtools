//! Backend trait contract tests.
//!
//! A minimal `TestBackend` implementation verifies that:
//! - the `Backend` trait is implementable,
//! - `RenderError` variants format correctly,
//! - `Capabilities::default()` returns the zero state.

mod helpers;
use helpers::empty_assets;

use tilemap_renderer::assets::Assets;
use tilemap_renderer::backend::{ Backend, Bitmap, Capabilities, Output, RenderError };
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

// Always returns errors — used for negative-path tests.
struct ErrorBackend
{
  load_error : Option< RenderError >,
  submit_error : Option< RenderError >,
  output_error : Option< RenderError >,
}

impl ErrorBackend
{
  fn load_missing( id : u32 ) -> Self
  {
    Self { load_error : Some( RenderError::MissingAsset( id ) ), submit_error : None, output_error : None }
  }

  fn load_backend_error( msg : &'static str ) -> Self
  {
    Self { load_error : Some( RenderError::BackendError( msg.into() ) ), submit_error : None, output_error : None }
  }

  fn submit_unsupported( what : &'static str ) -> Self
  {
    Self { load_error : None, submit_error : Some( RenderError::Unsupported( what ) ), output_error : None }
  }

  fn submit_missing( id : u32 ) -> Self
  {
    Self { load_error : None, submit_error : Some( RenderError::MissingAsset( id ) ), output_error : None }
  }

  fn output_backend_error( msg : &'static str ) -> Self
  {
    Self { load_error : None, submit_error : None, output_error : Some( RenderError::BackendError( msg.into() ) ) }
  }
}

impl Backend for ErrorBackend
{
  fn load_assets( &mut self, _assets : &Assets ) -> Result< (), RenderError >
  {
    if let Some( e ) = self.load_error.take() { return Err( e ); }
    Ok( () )
  }

  fn submit( &mut self, _commands : &[ RenderCommand ] ) -> Result< (), RenderError >
  {
    if let Some( e ) = self.submit_error.take() { return Err( e ); }
    Ok( () )
  }

  fn output( &self ) -> Result< Output, RenderError >
  {
    // output takes &self so we cannot take from an Option; store as a cell-style flag via a copy.
    match &self.output_error
    {
      Some( RenderError::BackendError( msg ) ) => Err( RenderError::BackendError( msg.clone() ) ),
      Some( RenderError::MissingAsset( id ) ) => Err( RenderError::MissingAsset( *id ) ),
      Some( RenderError::Unsupported( s ) ) => Err( RenderError::Unsupported( s ) ),
      None => Ok( Output::Presented ),
      _ => unreachable!( "add arm for new RenderError variant" ),
    }
  }

  fn resize( &mut self, _width : u32, _height : u32 )
  {
  }

  fn capabilities( &self ) -> Capabilities
  {
    Capabilities::default()
  }
}

// Returns Output::Bitmap from output().
struct BitmapBackend;

impl Backend for BitmapBackend
{
  fn load_assets( &mut self, _assets : &Assets ) -> Result< (), RenderError >
  {
    Ok( () )
  }

  fn submit( &mut self, _commands : &[ RenderCommand ] ) -> Result< (), RenderError >
  {
    Ok( () )
  }

  fn output( &self ) -> Result< Output, RenderError >
  {
    Ok( Output::Bitmap( Bitmap { bytes : vec![ 255u8; 4 ], width : 1, height : 1, channels : 4 } ) )
  }

  fn resize( &mut self, _width : u32, _height : u32 )
  {
  }

  fn capabilities( &self ) -> Capabilities
  {
    Capabilities::default()
  }
}

// Returns Output::Presented from output().
struct PresentedBackend;

impl Backend for PresentedBackend
{
  fn load_assets( &mut self, _assets : &Assets ) -> Result< (), RenderError >
  {
    Ok( () )
  }

  fn submit( &mut self, _commands : &[ RenderCommand ] ) -> Result< (), RenderError >
  {
    Ok( () )
  }

  fn output( &self ) -> Result< Output, RenderError >
  {
    Ok( Output::Presented )
  }

  fn resize( &mut self, _width : u32, _height : u32 )
  {
  }

  fn capabilities( &self ) -> Capabilities
  {
    Capabilities::default()
  }
}

// ============================================================================
// Tests
// ============================================================================

/// Verifies that `load_assets` succeeds on a valid empty asset set and
/// sets the internal `assets_loaded` flag, confirming the call reaches
/// the backend implementation.
#[ test ]
fn backend_load_assets_valid()
{
  let mut b = TestBackend::new();
  let assets = empty_assets();
  assert!( b.load_assets( &assets ).is_ok() );
  assert!( b.assets_loaded );
}

/// Verifies that `load_assets` accepts an empty `Assets` struct without error.
/// Empty asset sets are a common initial state and must not be rejected.
#[ test ]
fn backend_load_assets_empty()
{
  let mut b = TestBackend::new();
  assert!( b.load_assets( &empty_assets() ).is_ok() );
}

/// Verifies that `submit` with an empty command slice returns `Ok` and
/// records zero commands — backends must handle the no-op frame case.
#[ test ]
fn backend_submit_empty_slice()
{
  let mut b = TestBackend::new();
  assert!( b.submit( &[] ).is_ok() );
  assert_eq!( b.last_command_count, 0 );
}

/// Verifies that a single `Clear` command is accepted by `submit` and
/// that the backend records the correct command count.
#[ test ]
fn backend_submit_clear()
{
  use tilemap_renderer::commands::Clear;
  let mut b = TestBackend::new();
  let cmds = [ RenderCommand::Clear( Clear { color : [ 0.0, 0.0, 0.0, 1.0 ] } ) ];
  assert!( b.submit( &cmds ).is_ok() );
  assert_eq!( b.last_command_count, 1 );
}

/// Verifies that `output` can return `Output::String` and that the
/// string payload is correctly propagated from the backend.
#[ test ]
fn backend_output_returns_string()
{
  let b = TestBackend::new();
  match b.output().unwrap()
  {
    Output::String( s ) => assert_eq!( s, "test" ),
    other => panic!( "unexpected output: {other:?}" ),
  }
}

/// Verifies that `output` can return `Output::Bitmap` with correct
/// width, height, channel count, and byte buffer length.
#[ test ]
fn backend_output_returns_bitmap()
{
  match BitmapBackend.output().unwrap()
  {
    Output::Bitmap( bmp ) =>
    {
      assert_eq!( bmp.width, 1 );
      assert_eq!( bmp.height, 1 );
      assert_eq!( bmp.channels, 4 );
      assert_eq!( bmp.bytes.len(), 4 );
    }
    other => panic!( "expected Bitmap, got {other:?}" ),
  }
}

/// Verifies that `output` can return `Output::Presented`, covering
/// the GPU-presented swap-chain path.
#[ test ]
fn backend_output_returns_presented()
{
  match PresentedBackend.output().unwrap()
  {
    Output::Presented => {}
    other => panic!( "expected Presented, got {other:?}" ),
  }
}

/// Verifies that `RenderError::MissingAsset` formats to a string that
/// contains both the word "missing asset" and the asset id.
#[ test ]
fn render_error_missing_asset_display()
{
  let e = RenderError::MissingAsset( 42 );
  let s = format!( "{e}" );
  assert!( s.contains( "missing asset" ), "got: {s}" );
  assert!( s.contains( "42" ), "got: {s}" );
}

/// Verifies that `RenderError::Unsupported` formats to a string that
/// contains "unsupported" and the feature name.
#[ test ]
fn render_error_unsupported_display()
{
  let e = RenderError::Unsupported( "gradients" );
  let s = format!( "{e}" );
  assert!( s.contains( "unsupported" ), "got: {s}" );
  assert!( s.contains( "gradients" ), "got: {s}" );
}

/// Verifies that `RenderError::BackendError` formats to a string that
/// contains "backend error" and the inner message.
#[ test ]
fn render_error_backend_error_display()
{
  let e = RenderError::BackendError( "gpu lost".to_string() );
  let s = format!( "{e}" );
  assert!( s.contains( "backend error" ), "got: {s}" );
  assert!( s.contains( "gpu lost" ), "got: {s}" );
}

/// Verifies that `resize` stores the given dimensions and that they
/// can be read back from the backend's internal state.
#[ test ]
fn backend_resize_stores_dimensions()
{
  let mut b = TestBackend::new();
  b.resize( 800, 600 );
  assert_eq!( b.width, 800 );
  assert_eq!( b.height, 600 );
}

/// Verifies that `load_assets` propagates `RenderError::MissingAsset`
/// when the backend signals a missing asset during load.
#[ test ]
fn backend_load_assets_missing_asset_error()
{
  let mut b = ErrorBackend::load_missing( 7 );
  let err = b.load_assets( &empty_assets() ).unwrap_err();
  assert!( matches!( err, RenderError::MissingAsset( 7 ) ) );
}

/// Verifies that `load_assets` propagates `RenderError::BackendError`
/// when the backend encounters a generic failure during asset load.
#[ test ]
fn backend_load_assets_backend_error()
{
  let mut b = ErrorBackend::load_backend_error( "disk full" );
  let err = b.load_assets( &empty_assets() ).unwrap_err();
  assert!( matches!( err, RenderError::BackendError( _ ) ) );
}

/// Verifies that `submit` propagates `RenderError::Unsupported` when
/// the backend rejects an unsupported feature in the command stream.
#[ test ]
fn backend_submit_unsupported()
{
  let mut b = ErrorBackend::submit_unsupported( "gradients" );
  let err = b.submit( &[] ).unwrap_err();
  assert!( matches!( err, RenderError::Unsupported( "gradients" ) ) );
}

/// Verifies that `submit` propagates `RenderError::MissingAsset` when
/// the backend cannot resolve an asset referenced in a command.
#[ test ]
fn backend_submit_missing_asset()
{
  let mut b = ErrorBackend::submit_missing( 99 );
  let err = b.submit( &[] ).unwrap_err();
  assert!( matches!( err, RenderError::MissingAsset( 99 ) ) );
}

/// Verifies that `output` propagates `RenderError::BackendError` when
/// the backend fails to produce a frame.
#[ test ]
fn backend_output_backend_error()
{
  let b = ErrorBackend::output_backend_error( "gpu lost" );
  let err = b.output().unwrap_err();
  assert!( matches!( err, RenderError::BackendError( _ ) ) );
}

/// Verifies that `Capabilities::default()` returns all boolean fields
/// as `false` and `max_texture_size` as `0`, representing a backend
/// that advertises no optional feature support.
#[ test ]
fn backend_capabilities_default_all_false()
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

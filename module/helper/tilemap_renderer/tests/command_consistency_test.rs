//! Cross-backend command/capabilities consistency check: `none`/`svg`/`native`
//! (the three backends constructible without a live external device -- see
//! their own `*_backend_test.rs` files) each submit the same shared
//! `RenderCommand` fixtures, asserting every backend honors its own
//! `capabilities()` claim -- a family marked `true` is accepted, a family
//! marked `false` is rejected or gracefully skipped, but never panics.
//!
//! `svg` is excluded from the "unsupported family" case (T04):
//! `SvgBackend::capabilities()` has zero `false` boolean fields (every
//! family is `true` -- confirmed by `svg_backend_test.rs::capabilities_all_true`
//! and by direct read of `src/adapters/svg.rs`'s `capabilities()` body), so
//! there is no family whose rejection could be honestly tested there without
//! violating this task's own AF2 anti-faking check ("genuinely false in that
//! backend's own capabilities() output, not assumed"). `svg`'s own claim is
//! still fully exercised: T03 already proves it accepts a command from a
//! family it declares `true`.
//!
//! Whole file gated to the three backends it actually exercises: every
//! fixture and module below is used only by `none_backend`/`svg_backend`/
//! `native_backend`, so a build enabling none of them (e.g.
//! `adapter-terminal`/`adapter-webgl`/`adapter-webgpu` alone) would
//! otherwise trip dead-code-deny/unused-imports-deny on this file's
//! top-level fixtures with nothing left to compile them in.
#![ cfg( any( feature = "adapter-none", feature = "adapter-svg", all( feature = "adapter-native", not( target_arch = "wasm32" ) ) ) ) ]

use tilemap_renderer::commands::{ RenderCommand, Sprite };
use tilemap_renderer::types::{ BlendMode, ResourceId, Transform };

#[ cfg( any( feature = "adapter-none", all( feature = "adapter-native", not( target_arch = "wasm32" ) ) ) ) ]
use tilemap_renderer::commands::BeginPath;
#[ cfg( any( feature = "adapter-none", all( feature = "adapter-native", not( target_arch = "wasm32" ) ) ) ) ]
use tilemap_renderer::types::{ DashStyle, FillRef, LineCap, LineJoin };

/// Only `svg_backend`/`native_backend` below construct `Assets` via
/// `crate::helpers::empty_assets` -- gated to match, so an `adapter-none`-only
/// build (neither module compiled in) does not trip dead-code-deny on an
/// unused `helpers::empty_assets`.
#[ cfg( any( feature = "adapter-svg", all( feature = "adapter-native", not( target_arch = "wasm32" ) ) ) ) ]
mod helpers;

/// The `Sprite` fixture shared across every backend's T03 case -- `none`,
/// `svg`, and `native` all declare `sprites: true` in their own
/// `capabilities()`.
fn sprite_command() -> RenderCommand
{
  RenderCommand::Sprite( Sprite
  {
    transform : Transform::default(),
    sprite : ResourceId::new( 0 ),
    tint : [ 1.0, 1.0, 1.0, 1.0 ],
    blend : BlendMode::default(),
    clip : None,
  })
}

/// The `BeginPath` fixture shared across `none`/`native`'s T04 case -- both
/// backends declare `paths: false` in their own `capabilities()` (confirmed
/// by direct read of `src/adapters/none.rs` and `src/adapters/native.rs`).
/// Gated to its two actual consumers so an `adapter-svg`-only build (which
/// has no T04 case -- see this file's top doc comment) does not trip
/// dead-code-deny on an unused fixture.
#[ cfg( any( feature = "adapter-none", all( feature = "adapter-native", not( target_arch = "wasm32" ) ) ) ) ]
fn unsupported_path_command() -> RenderCommand
{
  RenderCommand::BeginPath( BeginPath
  {
    transform : Transform::default(),
    fill : FillRef::Solid( [ 0.0, 0.0, 0.0, 1.0 ] ),
    stroke_color : [ 0.0, 0.0, 0.0, 1.0 ],
    stroke_width : 1.0,
    stroke_cap : LineCap::Butt,
    stroke_join : LineJoin::Miter,
    stroke_dash : DashStyle::default(),
    blend : BlendMode::Normal,
    clip : None,
  })
}

#[ cfg( feature = "adapter-none" ) ]
mod none_backend
{
  use tilemap_renderer::adapters::none::NoneBackend;
  use tilemap_renderer::backend::Backend;
  use tilemap_renderer::types::RenderConfig;

  /// T03 -- `NoneBackend` declares `sprites: true`; submitting one `Sprite`
  /// returns `Ok`.
  #[ test ]
  fn sprite_command_returns_ok()
  {
    let mut backend = NoneBackend::new( RenderConfig::default() );
    assert!( backend.submit( &[ super::sprite_command() ] ).is_ok() );
  }

  /// T04 -- `NoneBackend` declares `paths: false`; submitting one
  /// `BeginPath` never panics. `NoneBackend::submit()` is an unconditional
  /// no-op `Ok` regardless of command content (confirmed by direct read of
  /// `src/adapters/none.rs`) -- the graceful-skip branch of T04's Expected
  /// Behavior.
  #[ test ]
  fn unsupported_family_command_does_not_panic()
  {
    let mut backend = NoneBackend::new( RenderConfig::default() );
    assert!( backend.submit( &[ super::unsupported_path_command() ] ).is_ok() );
  }
}

#[ cfg( feature = "adapter-svg" ) ]
mod svg_backend
{
  use tilemap_renderer::adapters::svg::SvgBackend;
  use tilemap_renderer::assets::{ Assets, ImageAsset, ImageSource, PixelFormat, SpriteAsset };
  use tilemap_renderer::backend::Backend;
  use tilemap_renderer::types::{ MipmapMode, RenderConfig, ResourceId, SamplerFilter, WrapMode };

  /// Builds an 8x8 solid-white `Assets` set -- one image, one sprite
  /// covering the full sheet -- so `ResourceId::new(0)` (this file's shared
  /// `sprite_command()` fixture) resolves to a real, loaded sprite instead
  /// of a dangling reference. Mirrors `native_backend`'s own
  /// `loaded_sprite_assets()`; needed since `Fix(BUG-209)` made
  /// `SvgBackend::cmd_sprite` return `RenderError::MissingAsset` for an
  /// unloaded sprite id instead of silently accepting it.
  fn loaded_sprite_assets() -> Assets
  {
    let mut assets = crate::helpers::empty_assets();
    assets.images.push( ImageAsset
    {
      id : ResourceId::new( 0 ),
      source : ImageSource::Bitmap
      {
        bytes : [ 255u8, 255, 255, 255 ].repeat( 8 * 8 ),
        width : 8,
        height : 8,
        format : PixelFormat::Rgba8,
      },
      filter : SamplerFilter::default(),
      mipmap : MipmapMode::default(),
      wrap : WrapMode::default(),
      premultiplied : false,
    });
    assets.sprites.push( SpriteAsset { id : ResourceId::new( 0 ), sheet : ResourceId::new( 0 ), region : [ 0.0, 0.0, 8.0, 8.0 ] } );
    assets
  }

  /// T03 -- `SvgBackend` declares every family `true` (including
  /// `sprites`); submitting one loaded `Sprite` returns `Ok`. See this
  /// file's own top doc comment for why `svg` has no T04 case.
  #[ test ]
  fn sprite_command_returns_ok()
  {
    let mut svg = SvgBackend::new( RenderConfig { width : 64, height : 64, ..Default::default() } );
    svg.assets_load( &loaded_sprite_assets() ).unwrap();
    assert!( svg.submit( &[ super::sprite_command() ] ).is_ok() );
  }
}

#[ cfg( all( feature = "adapter-native", not( target_arch = "wasm32" ) ) ) ]
mod native_backend
{
  use tilemap_renderer::adapters::native::NativeBackend;
  use tilemap_renderer::assets::{ Assets, ImageAsset, ImageSource, PixelFormat, SpriteAsset };
  use tilemap_renderer::backend::Backend;
  use tilemap_renderer::types::{ MipmapMode, RenderConfig, ResourceId, SamplerFilter, WrapMode };

  /// Builds an 8x8 solid-white `Assets` set -- one image, one sprite
  /// covering the full sheet -- so `ResourceId::new(0)` (this file's shared
  /// `sprite_command()` fixture) resolves to a real, loaded sprite instead
  /// of a dangling reference. Mirrors `native_backend_test.rs`'s own
  /// `solid_sprite_assets()`.
  fn loaded_sprite_assets() -> Assets
  {
    let mut assets = crate::helpers::empty_assets();
    assets.images.push( ImageAsset
    {
      id : ResourceId::new( 0 ),
      source : ImageSource::Bitmap
      {
        bytes : [ 255u8, 255, 255, 255 ].repeat( 8 * 8 ),
        width : 8,
        height : 8,
        format : PixelFormat::Rgba8,
      },
      filter : SamplerFilter::default(),
      mipmap : MipmapMode::default(),
      wrap : WrapMode::default(),
      premultiplied : false,
    });
    assets.sprites.push( SpriteAsset { id : ResourceId::new( 0 ), sheet : ResourceId::new( 0 ), region : [ 0.0, 0.0, 8.0, 8.0 ] } );
    assets
  }

  fn backend() -> NativeBackend
  {
    NativeBackend::new( RenderConfig { width : 64, height : 64, ..Default::default() } )
    .expect( "NativeBackend::new failed -- needs a Vulkan ICD (a software one such as lavapipe suffices)" )
  }

  /// T03 -- `NativeBackend` declares `sprites: true`; submitting one loaded
  /// `Sprite` returns `Ok`.
  #[ test ]
  fn sprite_command_returns_ok()
  {
    let mut backend = backend();
    backend.assets_load( &loaded_sprite_assets() ).expect( "assets_load failed" );
    assert!( backend.submit( &[ super::sprite_command() ] ).is_ok() );
  }

  /// T04 -- `NativeBackend` declares `paths: false`; submitting one
  /// `BeginPath` returns `Err`, never panics. Confirmed by direct read of
  /// `NativeBackend::submit()`'s match arms: any command other than a
  /// leading `Clear` or `Sprite` hits its `_ =>` arm, returning
  /// `RenderError::Unsupported` -- the reject branch of T04's Expected
  /// Behavior.
  #[ test ]
  fn unsupported_family_command_is_rejected_without_panic()
  {
    let mut backend = backend();
    backend.assets_load( &loaded_sprite_assets() ).expect( "assets_load failed" );
    assert!( backend.submit( &[ super::unsupported_path_command() ] ).is_err() );
  }
}

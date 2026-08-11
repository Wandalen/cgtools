//! `NoneBackend` no-op contract tests.
//!
//! Proves genuine no-op behavior (not merely that the trait compiles):
//! `NoneBackend` accepts any input and never surfaces a `RenderError`,
//! never inspects the commands or assets it is given, and always
//! reports the same `Presented` output and default `Capabilities`.

#![ cfg( feature = "adapter-none" ) ]

use tilemap_renderer::assets::*;
use tilemap_renderer::backend::*;
use tilemap_renderer::commands::*;
use tilemap_renderer::types::*;
use tilemap_renderer::adapters::none::NoneBackend;

mod helpers;
use helpers::empty_assets;

/// T01 -- `load_assets` accepts a non-empty `Assets` set and returns
/// `Ok(())`, storing nothing observable.
#[ test ]
fn load_assets_non_empty_returns_ok()
{
  let mut backend = NoneBackend::new( RenderConfig::default() );
  let mut assets = empty_assets();
  assets.geometries.push( GeometryAsset
  {
    id : ResourceId::new( 0 ),
    positions : Source::Bytes( vec![] ),
    uvs : None,
    indices : None,
    data_type : DataType::F32,
  });

  assert!( backend.load_assets( &assets ).is_ok() );
}

/// T02 -- `submit` accepts a non-empty command slice (a `Sprite` draw)
/// after `load_assets` and returns `Ok(())`.
#[ test ]
fn submit_non_empty_returns_ok()
{
  let mut backend = NoneBackend::new( RenderConfig::default() );
  backend.load_assets( &empty_assets() ).unwrap();

  let commands =
  [
    RenderCommand::Sprite( Sprite
    {
      transform : Transform::default(),
      sprite : ResourceId::new( 0 ),
      tint : [ 1.0, 1.0, 1.0, 1.0 ],
      blend : BlendMode::default(),
      clip : None,
    }),
  ];

  assert!( backend.submit( &commands ).is_ok() );
}

/// T03 -- `output` always returns `Ok(Output::Presented)` after a
/// `submit` call.
#[ test ]
fn output_always_presented_after_submit()
{
  let mut backend = NoneBackend::new( RenderConfig::default() );
  backend.submit( &[] ).unwrap();

  match backend.output()
  {
    Ok( Output::Presented ) => {}
    other => panic!( "expected Ok(Presented), got {other:?}" ),
  }
}

/// T04 -- `resize` never panics regardless of call ordering (before
/// `load_assets`, then again after `submit`), and `output()` called
/// immediately afterward still returns `Presented` -- proving `resize`
/// cannot leave the backend in a state that changes `output`'s return.
#[ test ]
fn resize_before_and_after_does_not_affect_output()
{
  let mut backend = NoneBackend::new( RenderConfig::default() );
  backend.resize( 800, 600 );
  backend.load_assets( &empty_assets() ).unwrap();
  backend.submit( &[] ).unwrap();
  backend.resize( 400, 300 );

  match backend.output()
  {
    Ok( Output::Presented ) => {}
    other => panic!( "expected Ok(Presented), got {other:?}" ),
  }
}

/// T05 / AF1 -- `capabilities()` on a freshly-constructed `NoneBackend`
/// matches `Capabilities::default()` field-for-field, compared against
/// a genuinely-constructed `Capabilities::default()` value (not a
/// hand-typed literal) on every field. Field-by-field is required here
/// rather than one whole-struct `assert_eq!` because `Capabilities` is
/// `#[non_exhaustive]` and does not derive `PartialEq` -- both are
/// pre-existing choices in `src/backend.rs` outside this task's scope
/// (task 084 Goal: exactly one new file, one `mod.rs` line, one
/// `Cargo.toml` line, and tests).
#[ test ]
fn capabilities_equals_default_field_for_field()
{
  let backend = NoneBackend::new( RenderConfig::default() );
  let actual = backend.capabilities();
  let expected = Capabilities::default();

  assert_eq!( actual.paths, expected.paths );
  assert_eq!( actual.text, expected.text );
  assert_eq!( actual.meshes, expected.meshes );
  assert_eq!( actual.sprites, expected.sprites );
  assert_eq!( actual.batches, expected.batches );
  assert_eq!( actual.gradients, expected.gradients );
  assert_eq!( actual.patterns, expected.patterns );
  assert_eq!( actual.clip_masks, expected.clip_masks );
  assert_eq!( actual.effects, expected.effects );
  assert_eq!( actual.blend_modes, expected.blend_modes );
  // `BlendMode` does not derive `PartialEq`, so the slice itself cannot be
  // compared via `assert_eq!`; `Capabilities::default()` is documented to
  // always be empty here, matching `backend_test.rs`'s own precedent.
  assert!( actual.supported_blend_modes.is_empty() );
  assert_eq!( actual.text_on_path, expected.text_on_path );
  assert_eq!( actual.max_texture_size, expected.max_texture_size );
}

/// AF2 -- `submit` never inspects command payloads: a `Sprite` command
/// referencing a resource id absent from the loaded (empty) `Assets`
/// still returns `Ok(())`, never `RenderError::MissingAsset` -- proving
/// commands are discarded, not resolved against loaded assets.
#[ test ]
fn submit_ignores_missing_asset_reference()
{
  let mut backend = NoneBackend::new( RenderConfig::default() );
  backend.load_assets( &empty_assets() ).unwrap();

  let commands =
  [
    RenderCommand::Sprite( Sprite
    {
      transform : Transform::default(),
      sprite : ResourceId::new( 999 ),
      tint : [ 1.0, 1.0, 1.0, 1.0 ],
      blend : BlendMode::default(),
      clip : None,
    }),
  ];

  assert!( backend.submit( &commands ).is_ok() );
}

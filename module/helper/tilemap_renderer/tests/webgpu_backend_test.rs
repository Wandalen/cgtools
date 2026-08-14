//! `WebGpuBackend` compile-and-construct-level contract tests.
//!
//! No test here constructs a live `Device`/canvas: `declared_capabilities`,
//! `sprite_draw_params`, and command-support classification are all pure
//! functions of their inputs, checkable without a browser WebGPU context --
//! this workspace does not yet have proven browser-runtime WebGPU test
//! infrastructure (see the governing task file's Out of Scope section).

#![ cfg( all( feature = "adapter-webgpu", target_arch = "wasm32" ) ) ]

use wasm_bindgen_test::wasm_bindgen_test;
use tilemap_renderer::commands::{ RenderCommand, Sprite, Clear };
use tilemap_renderer::types::{ Transform, ResourceId, BlendMode };
use tilemap_renderer::adapters::webgpu::WebGpuBackend;

/// T03 -- `declared_capabilities()` matches this task's honest subset:
/// sprites only, every other command family `false`.
#[ wasm_bindgen_test ]
fn declared_capabilities_matches_honest_subset()
{
  let capabilities = WebGpuBackend::declared_capabilities();

  assert!( capabilities.sprites, "sprites must be true -- submit actually translates them" );
  assert!( !capabilities.paths );
  assert!( !capabilities.text );
  assert!( !capabilities.meshes );
  assert!( !capabilities.batches );
  assert!( !capabilities.gradients );
  assert!( !capabilities.patterns );
  assert!( !capabilities.clip_masks );
  assert!( !capabilities.effects );
  assert!( !capabilities.blend_modes );
  assert!( capabilities.supported_blend_modes.is_empty() );
  assert!( !capabilities.text_on_path );
}

/// Builds a minimal `Sprite` command at `position`, otherwise default.
fn sprite_at( position : [ f32; 2 ], resource_id : u32 ) -> Sprite
{
  Sprite
  {
    transform : Transform { position, ..Transform::default() },
    sprite : ResourceId::new( resource_id ),
    tint : [ 1.0, 1.0, 1.0, 1.0 ],
    blend : BlendMode::default(),
    clip : None,
  }
}

/// T05 -- `sprite_draw_params` returns parameters that actually track its
/// input: two sprites with different positions/resource ids yield two
/// different results, not a hardcoded constant.
#[ wasm_bindgen_test ]
fn sprite_draw_params_differ_with_input()
{
  let a = sprite_at( [ 10.0, 20.0 ], 3 );
  let b = sprite_at( [ 99.0, -5.0 ], 7 );

  let ( position_a, id_a ) = WebGpuBackend::sprite_draw_params( &a );
  let ( position_b, id_b ) = WebGpuBackend::sprite_draw_params( &b );

  assert_eq!( position_a, [ 10.0, 20.0 ] );
  assert_eq!( id_a, 3 );
  assert_eq!( position_b, [ 99.0, -5.0 ] );
  assert_eq!( id_b, 7 );
  assert_ne!( position_a, position_b );
  assert_ne!( id_a, id_b );
}

/// AF2 -- a command family `declared_capabilities()` reports `false` for
/// must be rejected by the real gate `submit` calls, not silently accepted.
/// Calls `WebGpuBackend::command_classify` directly -- the exact function
/// `submit`'s loop invokes -- so this is the real anti-faking check, not a
/// decoy that could silently drift from the code path it claims to cover.
#[ wasm_bindgen_test ]
fn unsupported_command_family_is_rejected()
{
  let clear = RenderCommand::Clear( Clear { color : [ 0.0, 0.0, 0.0, 1.0 ] } );
  let sprite = RenderCommand::Sprite( sprite_at( [ 0.0, 0.0 ], 0 ) );

  // `Clear` is only honored as the *leading* command of a batch (consumed by
  // `submit` before its classification loop runs); mid-batch -- which is
  // exactly what `command_classify` itself sees -- it is, like every other
  // non-`Sprite` family, unsupported.
  assert!( WebGpuBackend::command_classify( &clear ).is_err() );
  assert!( WebGpuBackend::command_classify( &sprite ).is_ok() );
}

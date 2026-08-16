//! `WebGlBackend::declared_capabilities` pure-function tests -- the
//! compile-and-construct-level coverage every sibling adapter already has
//! (`none_backend_test.rs`, `svg_backend_test.rs`, `native_backend_test.rs`,
//! `webgpu_backend_test.rs`), extended to the one adapter that previously had
//! none. No `WebGl2RenderingContext`/`web_sys` call anywhere in this file --
//! see the governing task file's Out of Scope section for why a live-context
//! test is not attempted here.

#![ cfg( feature = "adapter-webgl" ) ]

use tilemap_renderer::adapters::webgl::WebGlBackend;
use tilemap_renderer::types::BlendMode;

/// T01 -- honest-subset pin: `meshes`/`sprites`/`batches` true, every other
/// boolean false, `supported_blend_modes` exactly `[Normal, Add, Multiply,
/// Screen]`.
#[ test ]
fn declared_capabilities_honest_subset()
{
  let caps = WebGlBackend::declared_capabilities( 4096 );

  assert!( !caps.paths );
  assert!( !caps.text );
  assert!( caps.meshes );
  assert!( caps.sprites );
  assert!( caps.batches );
  assert!( !caps.gradients );
  assert!( !caps.patterns );
  assert!( !caps.clip_masks );
  assert!( !caps.effects );
  assert!( !caps.blend_modes );
  assert!( !caps.text_on_path );
  assert_eq!( caps.max_texture_size, 4096 );
  assert!
  (
    matches!
    (
      caps.supported_blend_modes,
      [ BlendMode::Normal, BlendMode::Add, BlendMode::Multiply, BlendMode::Screen ]
    ),
    "supported_blend_modes did not match the expected 4-element honest subset: {:?}", caps.supported_blend_modes
  );
}

/// T02 / AF1 -- anti-hardcoding pin: two different `max_texture_size` inputs
/// produce two different outputs, each equal to its own input.
#[ test ]
fn declared_capabilities_max_texture_size_reflects_input()
{
  let small = WebGlBackend::declared_capabilities( 2048 );
  let large = WebGlBackend::declared_capabilities( 8192 );

  assert_eq!( small.max_texture_size, 2048 );
  assert_eq!( large.max_texture_size, 8192 );
  assert_ne!( small.max_texture_size, large.max_texture_size );
}

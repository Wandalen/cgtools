//! Shader assembly for the sun-grid-lines diagram: composes the reusable
//! WGSL chunks from the `shader_chunks` crate ( the fullscreen-triangle
//! vertex stage and the `hash21`/`value_noise`/`fbm3` noise stack ) ahead
//! of this example's own fragment-only body, owned by `scene.rhai`'s
//! `shader` field (see [`scene`](crate::scene)) rather than a separate
//! WGSL file. Kept free of any wasm/WebGPU dependency so assembly can be
//! unit-tested on the native target, same as the [`scene`](crate::scene)
//! loader.

/// Returns the complete WGSL shader source: the four `shader_chunks`
/// chunks — concatenated dependency-before-dependent by
/// `shader_chunks::compose()` — followed by `fragment_wgsl`. `fragment_wgsl`
/// is this scene's own, non-reusable fragment body ( `Uniforms`, the scene
/// constants, and `fs_main`, loaded from `scene.rhai`'s `shader` field via
/// [`scene::SceneConfig::load`](crate::scene::SceneConfig::load) ) — not
/// valid standalone WGSL on its own, since it consumes `VertexOutput`,
/// `hash21`, and `fbm3`, which the chunks provide.
#[ must_use ]
pub fn assemble( fragment_wgsl : &str ) -> String
{
  let chunks_wgsl = shader_chunks::compose
  (
    &[ shader_chunks::HASH21, shader_chunks::VALUE_NOISE, shader_chunks::FBM3, shader_chunks::FULLSCREEN_TRIANGLE ]
  );
  format!( "{chunks_wgsl}\n\n{fragment_wgsl}" )
}

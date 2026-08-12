//! Shader assembly for the sun-grid-lines diagram: composes the reusable
//! WGSL chunks from the `shader_chunks_core` crate ( the fullscreen-triangle
//! vertex stage and the `hash21`/`value_noise`/`fbm3` noise stack ) ahead
//! of this example's own fragment-only body, `FRAGMENT_WGSL` — loaded from
//! `shader/scene_fragment.wgsl` rather than `scene.rhai`, since the shader
//! is a program, not scene data. Kept free of any wasm/WebGPU dependency
//! so assembly can be unit-tested on the native target, same as the
//! `scene` module's loader.

/// This scene's own, non-reusable fragment body ( `Uniforms`, the scene
/// constants, and `fs_main` ) — not valid standalone WGSL on its own,
/// since it consumes `VertexOutput`, `hash21`, and `fbm3`, which
/// `assemble()`'s chunks provide.
pub const FRAGMENT_WGSL : &str = include_str!( "../shader/scene_fragment.wgsl" );

/// Returns the complete WGSL shader source: the four `shader_chunks_core`
/// chunks — concatenated dependency-before-dependent by
/// `shader_chunks_core::compose()` — followed by `FRAGMENT_WGSL`.
#[ must_use ]
pub fn assemble() -> String
{
  let chunks_wgsl = shader_chunks_core::compose
  (
    &[ shader_chunks_core::HASH21, shader_chunks_core::VALUE_NOISE, shader_chunks_core::FBM3, shader_chunks_core::FULLSCREEN_TRIANGLE ]
  );
  format!( "{chunks_wgsl}\n\n{FRAGMENT_WGSL}" )
}

//! Shader assembly for the sun-grid-lines diagram: selects the reusable
//! WGSL chunks this scene actually uses from `shader_chunks_core` ( the
//! fullscreen-triangle vertex stage and the `hash21`/`value_noise`/`fbm3`
//! noise stack — imported by name, at compile time ), defines the scene's
//! own fragment stage as a local chunk of the same shape, and composes the
//! whole mixed set dependency-before-dependent. Kept free of any
//! wasm/WebGPU dependency so assembly can be unit-tested on the native
//! target, same as the `scene` module's loader.

use shader_chunks_core::{ chunk, ChunkDescriptor };

/// This scene's own, non-reusable fragment stage ( `Uniforms`, the scene
/// constants, and `fs_main` ) as a locally-defined chunk — the same
/// [`ChunkDescriptor`] shape as the bundled rows, mirroring the `//@`
/// manifest at the top of `shader/scene_fragment.wgsl` ( the
/// `scene_fragment_descriptor_matches_its_manifest` test keeps the mirror
/// honest ). Not valid standalone WGSL: its `depends_on` names the chunks
/// providing `VertexOutput`, `hash21`, and `fbm3`.
pub const SCENE_FRAGMENT : ChunkDescriptor = ChunkDescriptor
{
  name : "scene_fragment",
  description : "Sun-grid-lines HUD scene fragment stage: animated star, orbit rings, planets, nebula, star field, grid, vignette.",
  tags : &[ ( "category", "scene" ) ],
  stage : Some( "fragment" ),
  depends_on : &[ "hash21", "fbm3", "fullscreen_triangle" ],
  exports : &[ "fn fs_main(in: VertexOutput) -> @location(0) vec4f" ],
  wgsl : include_str!( "../shader/scene_fragment.wgsl" ),
};

/// Every chunk this scene's shader is built from: the four bundled chunks
/// it uses, imported by name at compile time ( a typo'd name fails the
/// build ), plus the local [`SCENE_FRAGMENT`]. A deliberate selection, not
/// the whole bundled table — a chunk added to `shader_chunks_core` later
/// does not silently grow this shader.
pub const SCENE_CHUNKS : &[ ChunkDescriptor ] =
&[
  chunk( "hash21" ),
  chunk( "value_noise" ),
  chunk( "fbm3" ),
  chunk( "fullscreen_triangle" ),
  SCENE_FRAGMENT,
];

// Compile-time guarantee that the selection above is transitively complete
// — dropping `chunk( "value_noise" )` fails this assert at build time, not
// the first composed frame.
const _ : () = assert!
(
  shader_chunks_core::dependency_closed( SCENE_CHUNKS ),
  "SCENE_CHUNKS must contain every chunk its members' depends_on name"
);

/// Returns the complete WGSL shader source: [`SCENE_CHUNKS`] — bundled
/// imports and the local fragment chunk alike — concatenated
/// dependency-before-dependent by [`shader_chunks_core::set_compose`].
#[ must_use ]
pub fn assemble() -> String
{
  shader_chunks_core::set_compose( SCENE_CHUNKS )
}

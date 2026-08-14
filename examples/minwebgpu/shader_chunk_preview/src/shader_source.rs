//! Shader assembly for the shader_chunk_preview example: selects
//! `shader_chunks_core`'s noise stack ( fullscreen-triangle vertex stage,
//! `hash21`/`value_noise`/`fbm3` ) and composes it with this crate's own
//! local `preview_fragment` chunk -- the only chunk in the workspace ( as
//! of this writing ) carrying `//@ param:` tunable-parameter declarations,
//! since annotating a *bundled* `shader/*.wgsl` chunk was ruled out of
//! scope by decision Q-03 ( see
//! `module/shader/shader_chunks_params/readme.md` ). Kept free of any
//! wasm/WebGPU dependency so assembly and parameter discovery can both be
//! unit-tested on the native target -- mirrors
//! `examples/orrery/webgpu/src/shader_source.rs`'s own split.

use shader_chunks_core::{ chunk, ChunkDescriptor };

/// This example's own, non-reusable fragment stage -- `Params`, its
/// `//@ param:`-declared tunables, and `fs_main` -- as a locally-defined
/// chunk, the same [`ChunkDescriptor`] shape as the bundled rows. Not valid
/// standalone WGSL: its `depends_on` names the chunks providing
/// `VertexOutput`, `hash21`, and `fbm3`. The `//@` manifest at the top of
/// `shader/preview_fragment.wgsl` mirrors this descriptor; the
/// `preview_fragment_descriptor_matches_its_manifest` test keeps the two
/// honest.
pub const PREVIEW_FRAGMENT : ChunkDescriptor = ChunkDescriptor
{
  name : "preview_fragment",
  description : "Warped fbm3 noise field with live-tunable frequency, domain-warp strength, and brightness.",
  tags : &[ ( "category", "scene" ) ],
  stage : Some( "fragment" ),
  depends_on : &[ "hash21", "fbm3", "fullscreen_triangle" ],
  exports : &[ "fn fs_main(in: VertexOutput) -> @location(0) vec4f" ],
  wgsl : include_str!( "../shader/preview_fragment.wgsl" ),
};

/// Every chunk this example's shader is built from: the three bundled
/// chunks it uses, imported by name at compile time ( a typo'd name fails
/// the build ), plus the local [`PREVIEW_FRAGMENT`]. A deliberate
/// selection, not the whole bundled table.
pub const PREVIEW_CHUNKS : &[ ChunkDescriptor ] =
&[
  chunk( "hash21" ),
  chunk( "value_noise" ),
  chunk( "fbm3" ),
  chunk( "fullscreen_triangle" ),
  PREVIEW_FRAGMENT,
];

// Compile-time guarantee that the selection above is transitively complete
// -- dropping `chunk( "value_noise" )` fails this assert at build time, not
// the first composed frame.
const _ : () = assert!
(
  shader_chunks_core::dependency_closed( PREVIEW_CHUNKS ),
  "PREVIEW_CHUNKS must contain every chunk its members' depends_on name"
);

/// Returns the complete WGSL shader source: [`PREVIEW_CHUNKS`] -- bundled
/// imports and the local fragment chunk alike -- concatenated
/// dependency-before-dependent by [`shader_chunks_core::set_compose`].
#[ must_use ]
pub fn assemble() -> String
{
  shader_chunks_core::set_compose( PREVIEW_CHUNKS )
}

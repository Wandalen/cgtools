// Bundled-chunk data table, spliced into `mod private` by lib.rs's
// `include!`. Deliberately data-shaped — one `ChunkDescriptor` row ( every
// manifest field of `shader/<name>/<name>.wgsl`, restated as Rust data ) and
// one `match` arm per chunk, nothing else — so a build script can later
// parse the manifests and emit this file verbatim during compilation; until
// then, adding a chunk means adding its row and its arm here by hand, and
// the `chunks_table_*_match_each_manifest` tests hold every restated field
// equal to what the `parse_*` functions read from the manifest itself.

/// Every bundled chunk, one [`ChunkDescriptor`] row per
/// `shader/<name>/<name>.wgsl` file, in declaration order — enumerate it
/// directly, or pass rows' `wgsl` to [`compose`]/[`try_compose`].
pub static CHUNKS : &[ ChunkDescriptor ] =
&[
  ChunkDescriptor
  {
    name : "hash21",
    description : "Single-value hash of a 2D point into [0, 1).",
    tags : &[ ( "category", "hash" ) ],
    stage : None,
    depends_on : &[],
    exports : &[ "fn hash21(p: vec2f) -> f32" ],
    wgsl : include_str!( "../../../../shader/hash21/hash21.wgsl" ),
  },
  ChunkDescriptor
  {
    name : "value_noise",
    description : "Bilinear-interpolated value noise sampled at a 2D point, in [0, 1).",
    tags : &[ ( "category", "noise" ) ],
    stage : None,
    depends_on : &[ "hash21" ],
    exports : &[ "fn value_noise(p: vec2f) -> f32" ],
    wgsl : include_str!( "../../../../shader/value_noise/value_noise.wgsl" ),
  },
  ChunkDescriptor
  {
    name : "fbm3",
    description : "Fixed 3-octave fractal Brownian motion built on value_noise, in [0, 0.875].",
    tags : &[ ( "category", "noise" ), ( "technique", "fractal" ) ],
    stage : None,
    depends_on : &[ "value_noise" ],
    exports : &[ "fn fbm3(p: vec2f) -> f32" ],
    wgsl : include_str!( "../../../../shader/fbm3/fbm3.wgsl" ),
  },
  ChunkDescriptor
  {
    name : "fullscreen_triangle",
    description : "Fullscreen-triangle vertex stage: 3 vertices, no vertex buffer, vertex_index alone picks the corner.",
    tags : &[ ( "category", "vertex" ) ],
    stage : Some( "vertex" ),
    depends_on : &[],
    exports :
    &[
      "struct VertexOutput { position: vec4f, uv: vec2f }",
      "fn vs_main(vertex_index: u32) -> VertexOutput",
    ],
    wgsl : include_str!( "../../../../shader/fullscreen_triangle/fullscreen_triangle.wgsl" ),
  },
];

/// Resolves one bundled chunk by its manifest-declared name, in O(1) — a
/// static `match`, no table scan, no manifest parsing. Returns `None` for a
/// name not in [`CHUNKS`].
#[ must_use ]
pub fn chunk_get( name : &str ) -> Option< &'static ChunkDescriptor >
{
  match name
  {
    "hash21" => Some( &CHUNKS[ 0 ] ),
    "value_noise" => Some( &CHUNKS[ 1 ] ),
    "fbm3" => Some( &CHUNKS[ 2 ] ),
    "fullscreen_triangle" => Some( &CHUNKS[ 3 ] ),
    _ => None,
  }
}

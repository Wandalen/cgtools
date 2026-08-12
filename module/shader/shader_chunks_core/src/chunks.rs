// Bundled-chunk data table, spliced into `mod private` by lib.rs's
// `include!`. Deliberately data-shaped — one `ChunkDescriptor` row and one
// `match` arm per `shader/*.wgsl` file, nothing else — so a build script can
// later emit this file verbatim during compilation; until then, adding a
// chunk means adding its row and its arm here by hand.

/// Every bundled chunk, one [`ChunkDescriptor`] row per `shader/*.wgsl`
/// file, in declaration order — enumerate it directly, or pass rows' `wgsl`
/// to [`compose`]/[`try_compose`].
pub static CHUNKS : &[ ChunkDescriptor ] =
&[
  ChunkDescriptor { name : "hash21", wgsl : include_str!( "../../../../shader/hash21.wgsl" ) },
  ChunkDescriptor { name : "value_noise", wgsl : include_str!( "../../../../shader/value_noise.wgsl" ) },
  ChunkDescriptor { name : "fbm3", wgsl : include_str!( "../../../../shader/fbm3.wgsl" ) },
  ChunkDescriptor { name : "fullscreen_triangle", wgsl : include_str!( "../../../../shader/fullscreen_triangle.wgsl" ) },
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

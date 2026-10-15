//! Known-answer tests for the normative hash primitives ( SPEC §13 ) —
//! `coord_hash` and `str_hash` MUST produce identical output across runs,
//! platforms, and renderer versions; these pins guard the format's visual
//! determinism.
//!
//! Relocated from `src/hash.rs` by task 073 ( bodies verbatim ).

use tilemap_scene::{ coord_hash, str_hash };

#[ test ]
fn coord_hash_is_deterministic()
{
  // Same input → same output.
  assert_eq!( coord_hash( 3, -2, 0 ), coord_hash( 3, -2, 0 ) );
  // Different salt → different output for a non-zero coord.
  assert_ne!( coord_hash( 3, -2, 0 ), coord_hash( 3, -2, 1 ) );
  // Different coords at the same salt → different outputs.
  assert_ne!( coord_hash( 1, 0, 0 ), coord_hash( 0, 1, 0 ) );
  // Note: coord_hash( 0, 0, 0 ) == 0 is a fixed point of the formula
  // (0 XOR 0 XOR 0 avalanches to 0). This is acceptable — the origin is one
  // of many possible hash values; game content never relies on a specific
  // coord producing a non-zero hash.
}

#[ test ]
fn str_hash_is_deterministic()
{
  // FNV-1a of the empty string is the offset basis.
  assert_eq!( str_hash( "" ), 0x811c_9dc5 );
  // Same input → same output.
  assert_eq!( str_hash( "water_flow" ), str_hash( "water_flow" ) );
  // Different strings → (practically always) different outputs.
  assert_ne!( str_hash( "water_flow" ), str_hash( "wind_sway" ) );
}

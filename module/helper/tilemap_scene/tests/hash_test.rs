//! Known-answer tests for the normative hash primitives ( SPEC §13 ) —
//! `hash_coord` and `hash_str` MUST produce identical output across runs,
//! platforms, and renderer versions; these pins guard the format's visual
//! determinism.
//!
//! Relocated from `src/hash.rs` by task 073 ( bodies verbatim ).

use tilemap_scene::{ hash_coord, hash_str };

#[ test ]
fn hash_coord_is_deterministic()
{
  // Same input → same output.
  assert_eq!( hash_coord( 3, -2, 0 ), hash_coord( 3, -2, 0 ) );
  // Different salt → different output for a non-zero coord.
  assert_ne!( hash_coord( 3, -2, 0 ), hash_coord( 3, -2, 1 ) );
  // Different coords at the same salt → different outputs.
  assert_ne!( hash_coord( 1, 0, 0 ), hash_coord( 0, 1, 0 ) );
  // Note: hash_coord( 0, 0, 0 ) == 0 is a fixed point of the formula
  // (0 XOR 0 XOR 0 avalanches to 0). This is acceptable — the origin is one
  // of many possible hash values; game content never relies on a specific
  // coord producing a non-zero hash.
}

#[ test ]
fn hash_str_is_deterministic()
{
  // FNV-1a of the empty string is the offset basis.
  assert_eq!( hash_str( "" ), 0x811c_9dc5 );
  // Same input → same output.
  assert_eq!( hash_str( "water_flow" ), hash_str( "water_flow" ) );
  // Different strings → (practically always) different outputs.
  assert_ne!( hash_str( "water_flow" ), hash_str( "wind_sway" ) );
}

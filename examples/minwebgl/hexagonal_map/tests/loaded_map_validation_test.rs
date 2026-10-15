//! Regression tests verifying `main.rs`'s `loaded_map_sync` rejects an uploaded map whose tiles
//! reference a player/object index outside the current `Config`'s range, instead of assigning it
//! and panicking on the next render.
//!
//! `hexagonal_map` is a binary-only example crate (no `[lib]` target), so this test reads
//! `main.rs`'s own real source text via `include_str!` (to anchor against regression of the actual
//! fix), plus a hand-ported pure-Rust mirror of the fixed `map_tile_indices_in_range` validation
//! (using minimal local structs standing in for `core_game::Tile`/`Config`, since those types
//! themselves carry no wasm dependency but live in a binary crate's `src/`, unreachable from an
//! integration test without a `[lib]` target) to verify the boundary logic directly.

const MAIN_RS : &str = include_str!( "../src/main.rs" );

struct FakeTile
{
  owner_index : u32,
  object_index : Option< u32 >,
}

struct FakeConfig
{
  player_colors_len : usize,
  object_props_len : usize,
}

/// Mirrors `main.rs`'s fixed `map_tile_indices_in_range`.
fn tile_in_range( tile : &FakeTile, config : &FakeConfig ) -> bool
{
  let owner_in_range = ( tile.owner_index as usize ) < config.player_colors_len;
  let object_in_range = match tile.object_index
  {
    Some( object_index ) => ( object_index as usize ) < config.object_props_len,
    None => true,
  };
  owner_in_range && object_in_range
}

/// ## Root Cause
/// `loaded_map_sync` deserialized a dropped map JSON straight into `map` with no check that its
/// tiles' `owner_index`/`object_index` were in range for the currently-loaded `game_config` —
/// `serde_json::from_str::<Map>` succeeding only proves the JSON is well-typed, not that its index
/// fields are valid for a config it was never checked against. A hand-edited file, or a map saved
/// under a config with more players/objects than the one it's re-loaded into, passed straight
/// through.
///
/// ## Why Not Caught
/// Every map exercised during normal use is either the crate's own freshly-created default map or
/// one previously saved from the SAME session's `game_config` — both always in range by
/// construction. The out-of-range path only opens once a map file crosses a config boundary
/// (edited by hand, or saved under a differently-sized config), which nothing in ordinary demo
/// usage does.
///
/// ## Fix Applied
/// Added `map_tile_indices_in_range`, checked before assigning a deserialized map in
/// `loaded_map_sync`; an out-of-range map is now rejected with a console warning (matching the
/// function's existing malformed-JSON warning path) instead of being assigned.
///
/// ## Prevention
/// This test exercises the boundary logic directly (in range / owner out of range / object out of
/// range / `None` object index) and anchors the real fix's call sites and helper name in `main.rs`
/// via `include_str!`, catching a regression back to unconditional assignment.
///
/// ## Pitfall
/// A deserialize `Ok` proves only that the JSON is well-typed, not that its values are safe to use
/// as array indices against a DIFFERENT, independently-loaded piece of state — cross-check
/// user-supplied indices against the state they're about to index into, not just their own shape.
#[ test ]
fn bug_reproducer_bug_327_loaded_map_rejects_out_of_range_indices()
{
  assert!
  (
    MAIN_RS.contains( "fn map_tile_indices_in_range" ),
    "main.rs should validate a loaded map's tile indices before assigning it (BUG-327)"
  );
  assert!
  (
    MAIN_RS.contains( "map_tile_indices_in_range( &m, game_config )" ),
    "loaded_map_sync should check map_tile_indices_in_range before assigning the deserialized \
    map (BUG-327)"
  );

  let config = FakeConfig { player_colors_len : 2, object_props_len : 3 };

  let in_range = FakeTile { owner_index : 1, object_index : Some( 2 ) };
  assert!( tile_in_range( &in_range, &config ), "owner/object both within range should pass" );

  let no_object = FakeTile { owner_index : 0, object_index : None };
  assert!( tile_in_range( &no_object, &config ), "a tile with no object_index should never fail on it" );

  let owner_out_of_range = FakeTile { owner_index : 2, object_index : None };
  assert!
  (
    !tile_in_range( &owner_out_of_range, &config ),
    "owner_index == player_colors_len is out of range (0-indexed) and must be rejected"
  );

  let object_out_of_range = FakeTile { owner_index : 0, object_index : Some( 3 ) };
  assert!
  (
    !tile_in_range( &object_out_of_range, &config ),
    "object_index == object_props_len is out of range (0-indexed) and must be rejected"
  );
}

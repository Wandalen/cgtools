//! Basic tests for hexagonal grid example.
//!
//! `hexagonal_grid` is a binary-only example crate (no `[lib]` target), so these
//! tests exercise the `tiles_tools` coordinate types the example itself is built on
//! rather than the binary's own code.

use tiles_tools::coordinates::hexagonal::{ Axial, Coordinate, Pointy };
use std::collections::HashSet;
use std::collections::hash_map::DefaultHasher;
use std::hash::{ Hash, Hasher };

const MAIN_RS : &str = include_str!( "../src/main.rs" );
const MAIN_VERT : &str = include_str!( "../shaders/main.vert" );
const MAIN_FRAG : &str = include_str!( "../shaders/main.frag" );

/// Extracts every `uniform <type> <name>;` declaration's `<name>` from a GLSL source string.
fn shader_uniform_names( src : &str ) -> HashSet< String >
{
  src
  .split( ';' )
  .map( str::trim )
  .filter( | s | s.starts_with( "uniform " ) )
  .filter_map( | s | s.split_whitespace().last() )
  .map( str::to_string )
  .collect()
}

/// Extracts every `uniform_upload( "<name>"` call site's `<name>` from the example's Rust source.
fn rust_uniform_upload_names( src : &str ) -> Vec< String >
{
  src
  .split( "uniform_upload( \"" )
  .skip( 1 )
  .filter_map( | s | s.split( '"' ).next() )
  .map( str::to_string )
  .collect()
}

#[ test ]
fn test_coordinate_hash()
{
  // Create two identical coordinates
  let coord1 = Coordinate::< Axial, Pointy >::new( 1, 2 );
  let coord2 = Coordinate::< Axial, Pointy >::new( 1, 2 );

  // Create a different coordinate
  let coord3 = Coordinate::< Axial, Pointy >::new( 3, 4 );

  // Verify that identical coordinates produce the same hash
  let mut hasher1 = DefaultHasher::new();
  coord1.hash( &mut hasher1 );
  let hash1 = hasher1.finish();

  let mut hasher2 = DefaultHasher::new();
  coord2.hash( &mut hasher2 );
  let hash2 = hasher2.finish();

  assert_eq!( hash1, hash2, "Hashes for identical coordinates should match" );

  // Verify that different coordinates produce different hashes
  let mut hasher3 = DefaultHasher::new();
  coord3.hash( &mut hasher3 );
  let hash3 = hasher3.finish();

  assert_ne!( hash1, hash3, "Hashes for different coordinates should not match" );

  // Verify that the hash works correctly in a HashSet
  let mut set = HashSet::new();
  set.insert( coord1 );
  assert!( set.contains( &coord2 ), "HashSet should recognize identical coordinates" );
  assert!( !set.contains( &coord3 ), "HashSet should not recognize different coordinates" );
}

/// ## Root Cause
/// `pathfind_demo`'s path-drawing block (`main.rs`, drawing the a-star path in green) uploads
/// `scale` to a uniform named `"u_mvp"`, but `main.vert` only declares `u_zoom`/`u_rotation` and
/// `main.frag` only declares `u_color` — `u_mvp` does not exist in either shader stage.
///
/// ## Why Not Caught
/// WebGL's `uniformXfv` silently no-ops when given a location for a uniform name the active
/// program doesn't declare (`get_uniform_location` returns `None`, and this crate's
/// `uniform_upload` doesn't surface that as an error) — no compile error, no runtime panic, no
/// visual glitch, because the immediately preceding obstacle-drawing block in the same function
/// call already uploaded the identical `scale` value to the real `u_zoom` uniform moments
/// earlier, so the path hexagons render at the correct zoom by accident of call ordering.
///
/// ## Fix Applied
/// Changed `"u_mvp"` to `"u_zoom"`, matching every other `uniform_upload` call site for this
/// shader (grid_demo, painting_demo, and the obstacle-drawing block earlier in this same
/// function).
///
/// ## Prevention
/// This test parses `main.rs`'s and both shader files' real source text (`include_str!`) and
/// asserts every `uniform_upload` call site's uniform name is actually declared in one of the
/// two shader stages — catches any future typo'd/stale uniform name workspace-wide for this
/// crate, not just this one call site.
///
/// ## Pitfall
/// A wrong uniform name is invisible both to the compiler (it's a runtime string) and to the
/// running demo (WebGL treats it as a no-op, and a stale value from an earlier draw call in the
/// same frame can mask the effect entirely) — only a static name/declaration cross-check like
/// this one, not manual visual testing, reliably catches it.
#[ test ]
fn bug_reproducer_bug_326_uniform_upload_names_match_shader_declarations()
{
  let declared : HashSet< String > = shader_uniform_names( MAIN_VERT )
  .union( &shader_uniform_names( MAIN_FRAG ) )
  .cloned()
  .collect();
  assert!( declared.contains( "u_zoom" ), "sanity: u_zoom should be declared in main.vert" );
  assert!( declared.contains( "u_color" ), "sanity: u_color should be declared in main.frag" );

  let used = rust_uniform_upload_names( MAIN_RS );
  assert!( !used.is_empty(), "sanity: main.rs should contain uniform_upload call sites" );

  for name in &used
  {
    assert!
    (
      declared.contains( name ),
      "main.rs calls uniform_upload( \"{name}\" ) but neither main.vert nor main.frag declares a uniform named \"{name}\" — likely a typo'd/stale uniform name (BUG-326)"
    );
  }
}
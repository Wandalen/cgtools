//! ## Root Cause
//! A single `vao` bound position/point-size attributes from `vert_buffer2` and the color
//! attribute from `vert_buffer`, instead of each of the two complete (position+size+color)
//! datasets getting its own independent VAO. This crate's own readme states the demo shows
//! "switch[ing] between different vertex configurations with a single binding call" — a
//! single Frankenstein VAO demonstrates no such thing, and `vert_buffer`'s own position and
//! point-size fields were never read anywhere.
//!
//! ## Why Not Caught
//! The crate has no lib target or native test target at all (`main.rs`-only wasm binary);
//! the demo still compiled and rendered *something* (5 points, correctly colored from
//! `vert_buffer`, positioned/sized from `vert_buffer2`), so nothing crashed or errored —
//! only a source read against the readme's own stated purpose reveals the mismatch.
//!
//! ## Fix Applied (BUG-YYY)
//! Split into two independent VAOs (`vao` bound entirely to `vert_buffer`, `vao2` bound
//! entirely to `vert_buffer2`), each with its own 3 attribute_pointer calls against a single
//! buffer, and two `draw_arrays` calls (bind `vao` → draw 5 → bind `vao2` → draw 5).
//!
//! ## Prevention
//! This structural test parses `main.rs` (no lib target to unit-test the setup directly)
//! and asserts: two `gl::vao::create` calls exist, each VAO's three `attribute_pointer`
//! calls all reference the same buffer variable (not a mix of `vert_buffer`/`vert_buffer2`
//! within one VAO's setup block), and two `draw_arrays` calls exist.
//!
//! ## Pitfall
//! Don't collapse this back to one VAO/one draw call as a "simplification" — the two
//! complete, self-sufficient datasets (`vert_data`, `vert_data2`) are the whole point of
//! this crate: demonstrating VAO state switching, not just drawing points.

const MAIN_RS : &str = include_str!( "../src/main.rs" );

#[ test ]
fn two_independent_vaos_are_created()
{
  let count = MAIN_RS.matches( "gl::vao::create( &gl )?" ).count();
  assert_eq!( count, 2, "expected exactly 2 VAOs created (one per dataset), found {count}: {MAIN_RS}" );
}

#[ test ]
fn two_draw_calls_exist()
{
  let count = MAIN_RS.matches( "gl.draw_arrays( GL::POINTS, 0, 5 )" ).count();
  assert_eq!( count, 2, "expected exactly 2 draw_arrays(POINTS, 0, 5) calls, found {count}: {MAIN_RS}" );
}

#[ test ]
fn each_vao_setup_block_uses_a_single_consistent_buffer()
{
  // Extract each `gl.bind_vertex_array( Some( &vaoN ) ); ... gl.bind_vertex_array( None );`
  // setup block and assert all 3 attribute_pointer calls inside it reference the same
  // buffer identifier — catches a regression back to the mixed-buffer bug.
  for vao_name in [ "&vao )", "&vao2 )" ]
  {
    let marker = format!( "Some( {vao_name}" );
    let start = MAIN_RS.find( &marker )
    .unwrap_or_else( || panic!( "setup bind for {vao_name} not found" ) );
    let block_end = MAIN_RS[ start.. ].find( "bind_vertex_array( None )" )
    .map_or_else( || panic!( "no matching unbind found for {vao_name} setup block" ), | i | start + i );
    let block = &MAIN_RS[ start..block_end ];

    let uses_buffer = block.contains( "&vert_buffer )?" );
    let uses_buffer2 = block.contains( "&vert_buffer2 )?" );
    assert!(
      uses_buffer ^ uses_buffer2,
      "setup block for {vao_name} must reference exactly one of vert_buffer/vert_buffer2, not a mix: {block}"
    );
  }
}

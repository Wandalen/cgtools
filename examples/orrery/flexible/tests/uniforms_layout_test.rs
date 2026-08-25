//! Guards the two properties none of `native_render_test.rs`'s /
//! `vulkan_render_test.rs`'s pixel-landmark checks cover for every field:
//! that `uniforms.rs`'s `UniformsRaw` struct declares its fields in the same
//! order as its own `to_bytes()` pushes them, and that both match
//! `shader/scene_fragment.wgsl`'s `Uniforms` struct ( defined in the sibling
//! `orrery_webgpu` crate, whose shader this crate's `scene_render` also
//! targets ). The render tests only sample two pixels ( sun-disc center,
//! background corner ), so a same-shape field swap outside those two
//! regions -- e.g. `ring_colors`/`ring_params`, or `grid_color`/
//! `grid_params` -- compiles cleanly, preserves `to_bytes()`'s
//! length-only `debug_assert_eq!( .., 704, .. )`, and produces no render-test
//! failure, only a silently corrupted uniform buffer past the divergence
//! point.

/// Extracts field names, in declaration order, from a `struct <name>\n{ .. }`
/// block -- both Rust and WGSL use the same `field_name : type,` shape in
/// this codebase's spaced-colon style, so one parser covers both sides.
/// Comment, attribute, and blank lines are skipped; only lines containing a
/// top-level `:` contribute a field name.
fn struct_field_names( source : &str, exact_struct_name : &str ) -> Vec< String >
{
  // `\n` immediately after the name guards against a same-prefixed sibling
  // struct name ( not present in either source used here, but keeps this
  // helper safe to reuse verbatim ).
  let header = format!( "struct {exact_struct_name}\n" );
  let start = source.find( &header )
  .unwrap_or_else( || panic!( "`struct {exact_struct_name}` not found" ) );
  let body_start = source[ start.. ].find( '{' ).expect( "struct must have an opening brace" ) + start + 1;
  let body_end = source[ body_start.. ].find( "\n}" ).expect( "struct must have a closing brace" ) + body_start;
  let body = &source[ body_start..body_end ];

  body.lines()
  .map( str::trim )
  .filter( | line | !line.is_empty() && !line.starts_with( "//" ) && !line.starts_with( '#' ) )
  .filter_map( | line | line.split( ':' ).next() )
  .map( | name | name.trim().to_string() )
  .collect()
}

/// Extracts every `self.<field>` access, in order of appearance, from
/// `pub fn to_bytes( &self ) -> Vec< u8 > { .. }`'s body -- the exact order
/// fields land in the byte buffer, independent of the struct's own
/// declaration order above it in the source file.
fn to_bytes_field_order( source : &str ) -> Vec< String >
{
  let start = source.find( "pub fn to_bytes" ).expect( "`pub fn to_bytes` not found" );
  let body_start = source[ start.. ].find( '{' ).expect( "to_bytes must have a body" ) + start + 1;
  let body_end = source[ body_start.. ].find( "\n  }" ).expect( "to_bytes body must close" ) + body_start;
  let body = &source[ body_start..body_end ];

  let mut fields = Vec::new();
  let mut rest = body;
  while let Some( at ) = rest.find( "self." )
  {
    let after = &rest[ at + "self.".len().. ];
    let end = after.find( | c : char | !( c.is_alphanumeric() || c == '_' ) ).unwrap_or( after.len() );
    fields.push( after[ ..end ].to_string() );
    rest = &after[ end.. ];
  }
  fields
}

// test_kind: bug_reproducer(BUG-308)
/// RED proof (manually confirmed by transiently swapping the `grid_color`/
/// `grid_params` declaration lines in a scratch edit of `uniforms.rs` before
/// writing this fix, then reverting): with the pre-fix codebase ( no test
/// tying the struct's declaration order, `to_bytes()`'s push order, and the
/// WGSL order together ), the swap compiles, `to_bytes()`'s 704-byte
/// `debug_assert_eq!` still holds ( total length is unchanged by a
/// same-shape swap ), and both existing render tests still pass ( neither
/// sampled pixel depends on the grid ) -- nothing catches it. This test
/// closes that gap for the full 27-field struct.
#[ test ]
fn uniforms_raw_order_matches_to_bytes_order_matches_wgsl_order()
{
  let rust_source = include_str!( "../src/uniforms.rs" );
  let wgsl_source = orrery_webgpu::shader_source::SCENE_FRAGMENT.wgsl;

  let struct_fields = struct_field_names( rust_source, "UniformsRaw" );
  let bytes_fields = to_bytes_field_order( rust_source );
  let wgsl_fields = struct_field_names( wgsl_source, "Uniforms" );

  assert_eq!
  (
    struct_fields, bytes_fields,
    "`UniformsRaw`'s declared field order must exactly match `to_bytes()`'s push order -- \
    a mismatch silently serializes fields into the wrong byte offsets"
  );
  assert_eq!
  (
    bytes_fields, wgsl_fields,
    "`to_bytes()`'s push order must exactly match `Uniforms`'s WGSL field order -- \
    a mismatch silently corrupts every uniform value after the divergence point"
  );
  assert!( struct_fields.len() > 20, "sanity check: the field-extraction parser must find the full ~27-field struct, not stop early" );
}

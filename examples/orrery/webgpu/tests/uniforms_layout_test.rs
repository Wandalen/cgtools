//! Guards the one property none of `shader_source_test.rs`'s existing
//! checks cover: that `uniforms.rs`'s `#[repr(C)]` `UniformsRaw` struct
//! declares its fields in the exact same order as `shader/scene_fragment.wgsl`'s
//! `Uniforms` struct. The two are never compared by the compiler — `UniformsRaw`
//! is uploaded to the GPU as raw bytes via `bytemuck::Pod`, and WGSL reads
//! those same bytes back through its own, independently-declared struct. A
//! reorder on either side compiles cleanly and produces no runtime error;
//! every field after the divergence point is silently read as the wrong
//! value ( e.g. `disc_params` bytes interpreted as `ring_colors` ), corrupting
//! the rendered frame with no panic and no validation failure to catch it.

/// Extracts field names, in declaration order, from a Rust- or WGSL-style
/// `struct <name>\n{ ... }` block -- both languages use the same
/// `field_name : type,` shape in this codebase's spaced-colon style, so one
/// parser covers both sides. Comment lines ( `//`, `///` ), attribute lines
/// ( `#[...]` ), and the struct header/braces themselves are skipped; only
/// lines containing a top-level `:` contribute a field name.
fn struct_field_names( source : &str, exact_struct_name : &str ) -> Vec< String >
{
  // `\n` immediately after the name excludes `UniformsRaw` from a
  // `struct Uniforms` search -- the two names share every character up to
  // where `Raw` continues instead of a newline.
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

// test_kind: bug_reproducer(BUG-307)
/// RED proof (manually confirmed by transiently swapping two field lines
/// in a scratch copy of `uniforms.rs` before writing this fix): with the
/// pre-fix codebase ( no test at all touching `UniformsRaw` field order ),
/// swapping e.g. `disc_params`/`ring_colors` compiles and passes every
/// existing test in this crate -- nothing catches it. This test closes
/// that gap.
#[ test ]
fn uniforms_raw_field_order_matches_wgsl_uniforms_struct()
{
  let rust_source = include_str!( "../src/uniforms.rs" );
  let wgsl_source = orrery_webgpu::shader_source::SCENE_FRAGMENT.wgsl;

  let rust_fields = struct_field_names( rust_source, "UniformsRaw" );
  let wgsl_fields = struct_field_names( wgsl_source, "Uniforms" );

  assert_eq!
  (
    rust_fields, wgsl_fields,
    "`UniformsRaw`'s Rust field order must exactly match `Uniforms`'s WGSL field order -- \
    a mismatch silently corrupts every uniform value after the divergence point, since the \
    struct is uploaded as raw bytes with no per-field validation on either side"
  );
  assert!( rust_fields.len() > 20, "sanity check: the field-extraction parser must find the full ~27-field struct, not stop early" );
}

//! Tests for [`discover`] and [`discover_chunk`] — `//@ param:` line
//! parsing, file-order/empty-input handling, malformed-directive panics,
//! and declared-range precedence over inference. All fixtures are
//! self-contained WGSL strings owned by this file — no real bundled
//! `shader/*.wgsl` chunk is read or annotated.

use shader_chunks_params::
{
  discover, discover_chunk, Parameter, ParameterKind, Range, RangeSource, ValueType,
};

#[ test ]
fn discover_parses_declared_range_argument_u32()
{
  let wgsl = "//@ param: octaves argument u32 range(1, 8)\n";
  let expected = vec!
  [
    Parameter
    {
      name : "octaves".to_string(),
      kind : ParameterKind::Argument,
      value_type : ValueType::U32,
      range : Some( ( Range { min : 1.0, max : 8.0 }, RangeSource::Declared ) ),
    },
  ];
  assert_eq!( discover( wgsl ), expected );
}

#[ test ]
fn discover_infers_range_for_define_kind_via_seed_name_pattern()
{
  let wgsl = "//@ param: seed define u32\n";
  let expected = vec!
  [
    Parameter
    {
      name : "seed".to_string(),
      kind : ParameterKind::Define,
      value_type : ValueType::U32,
      range : Some( ( Range { min : 0.0, max : 65535.0 }, RangeSource::Inferred ) ),
    },
  ];
  assert_eq!( discover( wgsl ), expected );
}

#[ test ]
fn discover_parses_declared_range_for_all_five_kinds()
{
  let wgsl = "\
//@ param: a argument u32 range(1, 2)
//@ param: b define i32 range(3, 4)
//@ param: c uniform f32 range(5, 6)
//@ param: d attribute vec2f range(7, 8)
//@ param: e texture texture_2d range(9, 10)
";
  let expected = vec!
  [
    Parameter { name : "a".to_string(), kind : ParameterKind::Argument, value_type : ValueType::U32, range : Some( ( Range { min : 1.0, max : 2.0 }, RangeSource::Declared ) ) },
    Parameter { name : "b".to_string(), kind : ParameterKind::Define, value_type : ValueType::I32, range : Some( ( Range { min : 3.0, max : 4.0 }, RangeSource::Declared ) ) },
    Parameter { name : "c".to_string(), kind : ParameterKind::Uniform, value_type : ValueType::F32, range : Some( ( Range { min : 5.0, max : 6.0 }, RangeSource::Declared ) ) },
    Parameter { name : "d".to_string(), kind : ParameterKind::Attribute, value_type : ValueType::Vec2F, range : Some( ( Range { min : 7.0, max : 8.0 }, RangeSource::Declared ) ) },
    Parameter { name : "e".to_string(), kind : ParameterKind::Texture, value_type : ValueType::Texture2d, range : Some( ( Range { min : 9.0, max : 10.0 }, RangeSource::Declared ) ) },
  ];
  assert_eq!( discover( wgsl ), expected );
}

#[ test ]
fn discover_infers_range_for_argument_kind_with_no_declared_range()
{
  let wgsl = "//@ param: radius argument f32\n";
  let expected = vec!
  [
    Parameter
    {
      name : "radius".to_string(),
      kind : ParameterKind::Argument,
      value_type : ValueType::F32,
      range : Some( ( Range { min : 0.0, max : 100.0 }, RangeSource::Inferred ) ),
    },
  ];
  assert_eq!( discover( wgsl ), expected );
}

#[ test ]
fn discover_returns_multiple_params_in_file_order()
{
  let wgsl = "\
//@ param: first argument u32 range(1, 2)
//@ param: second uniform f32 range(3, 4)
";
  let result = discover( wgsl );
  assert_eq!( result.len(), 2 );
  assert_eq!( result[ 0 ].name, "first" );
  assert_eq!( result[ 1 ].name, "second" );
}

#[ test ]
fn discover_returns_empty_vec_when_no_param_lines()
{
  let wgsl = "//@ name: no_params\n//@ description: has no tunable parameters\n";
  assert_eq!( discover( wgsl ), Vec::< Parameter >::new() );
}

#[ test ]
#[ should_panic( expected = "unknown WGSL type token" ) ]
fn discover_panics_on_unknown_type_token()
{
  let _ = discover( "//@ param: x argument bogus_type\n" );
}

#[ test ]
#[ should_panic( expected = "unknown kind token" ) ]
fn discover_panics_on_unknown_kind_token()
{
  let _ = discover( "//@ param: x bogus_kind u32\n" );
}

#[ test ]
#[ should_panic( expected = "malformed `//@ param:` line" ) ]
fn discover_panics_on_wrong_token_count()
{
  let _ = discover( "//@ param: x argument\n" );
}

#[ test ]
fn discover_declared_range_overrides_name_pattern_inference()
{
  let wgsl = "//@ param: octaves argument u32 range(1, 8)\n";
  let result = discover( wgsl );
  assert_eq!( result[ 0 ].range, Some( ( Range { min : 1.0, max : 8.0 }, RangeSource::Declared ) ) );
}

#[ test ]
fn discover_declared_range_overrides_type_fallback_inference()
{
  // "workgroup_x" matches no name pattern, so a `u32` param with no
  // declared range would infer the type-fallback `[0, 16]` ( see
  // `infer_range_attribute_workgroup_x_falls_through_to_type_fallback` in
  // `range_inference_test.rs` ). Declaring `range(2, 4)` here must win
  // outright rather than blend with or defer to that fallback.
  let wgsl = "//@ param: workgroup_x attribute u32 range(2, 4)\n";
  let result = discover( wgsl );
  assert_eq!( result[ 0 ].range, Some( ( Range { min : 2.0, max : 4.0 }, RangeSource::Declared ) ) );
}

const LOCAL_GLOW_WGSL : &str = "\
//@ name: glow
//@ description: Doubled value noise, a test-local chunk.
//@ tags: category:test
//@ depends_on: value_noise
//@ export: fn glow(p: vec2f) -> f32
//@ param: octaves argument u32 range(1, 8)
//@ param: seed define u32

fn glow( p : vec2f, octaves : u32, seed : u32 ) -> f32
{
  return value_noise( p ) * 2.0;
}
";

const LOCAL_GLOW : shader_chunks_core::ChunkDescriptor = shader_chunks_core::ChunkDescriptor
{
  name : "glow",
  description : "Doubled value noise, a test-local chunk.",
  tags : &[ ( "category", "test" ) ],
  stage : None,
  depends_on : &[ "value_noise" ],
  exports : &[ "fn glow(p: vec2f) -> f32" ],
  wgsl : LOCAL_GLOW_WGSL,
};

#[ test ]
fn discover_chunk_matches_discover_on_wgsl_field()
{
  assert_eq!( discover_chunk( &LOCAL_GLOW ), discover( LOCAL_GLOW_WGSL ) );
}

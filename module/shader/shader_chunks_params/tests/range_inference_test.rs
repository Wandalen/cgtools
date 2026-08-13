//! Tests for [`infer_range`] — every name-pattern rule and every
//! WGSL-type-fallback rule from `docs/algorithm/001_range_inference_heuristic.md`,
//! plus the `kind == Texture` / `value_type == Bool` short-circuits that
//! always return `None` regardless of name.

use shader_chunks_params::{ infer_range, ParameterKind, Range, ValueType };

#[ test ]
fn infer_range_amplitude_name_pattern_matches_declared_scenario()
{
  let result = infer_range( ParameterKind::Uniform, ValueType::F32, "amplitude" );
  assert_eq!( result, Some( Range { min : 0.0, max : 1.0 } ) );
}

#[ test ]
fn infer_range_name_pattern_takes_precedence_over_type_fallback()
{
  // `u32`'s own type-fallback is `[0, 16]` ( see
  // `infer_range_type_fallback_u32_and_vec_u_variants` below ); "amplitude"
  // must still win with `[0, 1]`, proving the name-pattern stage runs
  // first and short-circuits before type-fallback is even consulted.
  let result = infer_range( ParameterKind::Uniform, ValueType::U32, "amplitude" );
  assert_eq!( result, Some( Range { min : 0.0, max : 1.0 } ) );
}

#[ test ]
fn infer_range_attribute_workgroup_x_falls_through_to_type_fallback()
{
  let result = infer_range( ParameterKind::Attribute, ValueType::U32, "workgroup_x" );
  assert_eq!( result, Some( Range { min : 0.0, max : 16.0 } ) );
}

#[ test ]
fn infer_range_texture_kind_is_always_none()
{
  assert_eq!( infer_range( ParameterKind::Texture, ValueType::Texture2d, "albedo" ), None );
  // Even a name that would otherwise pattern-match ( "radius" ) must still
  // yield `None` — the texture-kind short-circuit runs before any
  // name-pattern lookup.
  assert_eq!( infer_range( ParameterKind::Texture, ValueType::Texture2d, "radius" ), None );
}

#[ test ]
fn infer_range_bool_type_is_always_none()
{
  assert_eq!( infer_range( ParameterKind::Uniform, ValueType::Bool, "enabled" ), None );
  // Same short-circuit precedence check as the texture-kind case above,
  // but for the bool-type rule.
  assert_eq!( infer_range( ParameterKind::Uniform, ValueType::Bool, "radius" ), None );
}

#[ test ]
fn infer_range_octaves_count_steps_iterations_pattern()
{
  let expected = Some( Range { min : 1.0, max : 8.0 } );
  assert_eq!( infer_range( ParameterKind::Argument, ValueType::U32, "octaves" ), expected );
  assert_eq!( infer_range( ParameterKind::Argument, ValueType::U32, "count" ), expected );
  assert_eq!( infer_range( ParameterKind::Argument, ValueType::U32, "steps" ), expected );
  assert_eq!( infer_range( ParameterKind::Argument, ValueType::U32, "iterations" ), expected );
}

#[ test ]
fn infer_range_seed_pattern()
{
  let result = infer_range( ParameterKind::Define, ValueType::U32, "seed" );
  assert_eq!( result, Some( Range { min : 0.0, max : 65535.0 } ) );
}

#[ test ]
fn infer_range_angle_rotation_pattern()
{
  let expected = Some( Range { min : 0.0, max : std::f64::consts::TAU } );
  assert_eq!( infer_range( ParameterKind::Uniform, ValueType::F32, "angle" ), expected );
  assert_eq!( infer_range( ParameterKind::Uniform, ValueType::F32, "rotation" ), expected );
}

#[ test ]
fn infer_range_scale_frequency_freq_pattern()
{
  let expected = Some( Range { min : 0.1, max : 10.0 } );
  assert_eq!( infer_range( ParameterKind::Uniform, ValueType::F32, "scale" ), expected );
  assert_eq!( infer_range( ParameterKind::Uniform, ValueType::F32, "frequency" ), expected );
  assert_eq!( infer_range( ParameterKind::Uniform, ValueType::F32, "freq" ), expected );
}

#[ test ]
fn infer_range_radius_size_width_height_pattern()
{
  let expected = Some( Range { min : 0.0, max : 100.0 } );
  assert_eq!( infer_range( ParameterKind::Argument, ValueType::F32, "radius" ), expected );
  assert_eq!( infer_range( ParameterKind::Argument, ValueType::F32, "size" ), expected );
  assert_eq!( infer_range( ParameterKind::Argument, ValueType::F32, "width" ), expected );
  assert_eq!( infer_range( ParameterKind::Argument, ValueType::F32, "height" ), expected );
}

#[ test ]
fn infer_range_type_fallback_u32_and_vec_u_variants()
{
  let expected = Some( Range { min : 0.0, max : 16.0 } );
  assert_eq!( infer_range( ParameterKind::Uniform, ValueType::U32, "value" ), expected );
  assert_eq!( infer_range( ParameterKind::Uniform, ValueType::Vec2U, "value" ), expected );
  assert_eq!( infer_range( ParameterKind::Uniform, ValueType::Vec3U, "value" ), expected );
  assert_eq!( infer_range( ParameterKind::Uniform, ValueType::Vec4U, "value" ), expected );
}

#[ test ]
fn infer_range_type_fallback_i32_and_vec_i_variants()
{
  let expected = Some( Range { min : -16.0, max : 16.0 } );
  assert_eq!( infer_range( ParameterKind::Uniform, ValueType::I32, "value" ), expected );
  assert_eq!( infer_range( ParameterKind::Uniform, ValueType::Vec2I, "value" ), expected );
  assert_eq!( infer_range( ParameterKind::Uniform, ValueType::Vec3I, "value" ), expected );
  assert_eq!( infer_range( ParameterKind::Uniform, ValueType::Vec4I, "value" ), expected );
}

#[ test ]
fn infer_range_type_fallback_f32_and_vec_f_variants()
{
  let expected = Some( Range { min : 0.0, max : 1.0 } );
  assert_eq!( infer_range( ParameterKind::Uniform, ValueType::F32, "value" ), expected );
  assert_eq!( infer_range( ParameterKind::Uniform, ValueType::Vec2F, "value" ), expected );
  assert_eq!( infer_range( ParameterKind::Uniform, ValueType::Vec3F, "value" ), expected );
  assert_eq!( infer_range( ParameterKind::Uniform, ValueType::Vec4F, "value" ), expected );
}

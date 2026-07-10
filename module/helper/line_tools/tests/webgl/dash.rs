
use super::*;
use line_tools::d3;

// === DashPattern default ===

#[ test ]
fn test_dash_pattern_default()
{
  let pattern = d3::DashPattern::default();
  match pattern
  {
    d3::DashPattern::V1( v ) => assert_eq!( v, 0.5 ),
    _ => panic!( "Default DashPattern should be V1" ),
  }
}

// === dash_use ===

#[ test ]
fn test_dash_use_enables_dash_define()
{
  let mut line = d3::Line::default();
  line.dash_use( true );

  let defines = line.defines_get();
  assert!( defines.contains( "#define USE_DASH\n" ) );
}

#[ test ]
fn test_dash_use_false_no_dash_define()
{
  let mut line = d3::Line::default();
  line.dash_use( false );

  let defines = line.defines_get();
  assert!( !defines.contains( "#define USE_DASH" ) );
}

#[ test ]
fn test_dash_use_toggle()
{
  let mut line = d3::Line::default();

  line.dash_use( true );
  assert!( line.defines_get().contains( "#define USE_DASH\n" ) );

  line.dash_use( false );
  assert!( !line.defines_get().contains( "#define USE_DASH" ) );
}

// === dash_pattern_set ===

#[ test ]
fn test_dash_pattern_set_v1()
{
  let mut line = d3::Line::default();
  line.dash_use( true );
  line.dash_pattern_set( d3::DashPattern::V1( 1.0 ) );

  let defines = line.defines_get();
  assert!( defines.contains( "#define USE_DASH_V1\n" ) );
  assert!( !defines.contains( "#define USE_DASH_V2" ) );
  assert!( !defines.contains( "#define USE_DASH_V3" ) );
  assert!( !defines.contains( "#define USE_DASH_V4" ) );
}

#[ test ]
fn test_dash_pattern_set_v2()
{
  let mut line = d3::Line::default();
  line.dash_use( true );
  line.dash_pattern_set( d3::DashPattern::V2( [ 0.5, 0.5 ] ) );

  let defines = line.defines_get();
  assert!( defines.contains( "#define USE_DASH_V2\n" ) );
  assert!( !defines.contains( "#define USE_DASH_V1" ) );
}

#[ test ]
fn test_dash_pattern_set_v3()
{
  let mut line = d3::Line::default();
  line.dash_use( true );
  line.dash_pattern_set( d3::DashPattern::V3( [ 0.5, 0.25, 0.25 ] ) );

  let defines = line.defines_get();
  assert!( defines.contains( "#define USE_DASH_V3\n" ) );
}

#[ test ]
fn test_dash_pattern_set_v4()
{
  let mut line = d3::Line::default();
  line.dash_use( true );
  line.dash_pattern_set( d3::DashPattern::V4( [ 0.4, 0.1, 0.4, 0.1 ] ) );

  let defines = line.defines_get();
  assert!( defines.contains( "#define USE_DASH_V4\n" ) );
}

#[ test ]
fn test_dash_pattern_set_same_variant_no_defines_change()
{
  let mut line = d3::Line::default();
  line.dash_use( true );

  // Default is V1, setting V1 again with different value
  // should not change the defines output since discriminant is the same
  line.dash_pattern_set( d3::DashPattern::V1( 0.5 ) );
  let defines1 = line.defines_get();

  line.dash_pattern_set( d3::DashPattern::V1( 0.8 ) );
  let defines2 = line.defines_get();

  assert_eq!( defines1, defines2 );
}

#[ test ]
fn test_dash_pattern_set_different_variant_changes_defines()
{
  let mut line = d3::Line::default();
  line.dash_use( true );
  line.dash_pattern_set( d3::DashPattern::V1( 0.5 ) );

  let defines1 = line.defines_get();
  assert!( defines1.contains( "#define USE_DASH_V1\n" ) );

  line.dash_pattern_set( d3::DashPattern::V2( [ 0.3, 0.7 ] ) );

  let defines2 = line.defines_get();
  assert!( defines2.contains( "#define USE_DASH_V2\n" ) );
  assert!( !defines2.contains( "#define USE_DASH_V1" ) );
}

// === defines_get combinations ===

#[ test ]
fn test_get_defines_empty_by_default()
{
  let line = d3::Line::default();
  let defines = line.defines_get();
  assert_eq!( defines, "" );
}

#[ test ]
fn test_get_defines_vertex_color()
{
  let mut line = d3::Line::default();
  line.vertex_color_use( true );

  let defines = line.defines_get();
  assert!( defines.contains( "#define USE_VERTEX_COLORS\n" ) );
}

#[ test ]
fn test_get_defines_alpha_to_coverage()
{
  let mut line = d3::Line::default();
  line.alpha_to_coverage_use( true );

  let defines = line.defines_get();
  assert!( defines.contains( "#define USE_ALPHA_TO_COVERAGE\n" ) );
}

#[ test ]
fn test_get_defines_world_units()
{
  let mut line = d3::Line::default();
  line.world_units_use( true );

  let defines = line.defines_get();
  assert!( defines.contains( "#define USE_WORLD_UNITS\n" ) );
}

#[ test ]
fn test_get_defines_all_flags()
{
  let mut line = d3::Line::default();
  line.vertex_color_use( true );
  line.alpha_to_coverage_use( true );
  line.world_units_use( true );
  line.dash_use( true );
  line.dash_pattern_set( d3::DashPattern::V3( [ 0.5, 0.25, 0.25 ] ) );

  let defines = line.defines_get();
  assert!( defines.contains( "#define USE_VERTEX_COLORS\n" ) );
  assert!( defines.contains( "#define USE_ALPHA_TO_COVERAGE\n" ) );
  assert!( defines.contains( "#define USE_WORLD_UNITS\n" ) );
  assert!( defines.contains( "#define USE_DASH\n" ) );
  assert!( defines.contains( "#define USE_DASH_V3\n" ) );
}

#[ test ]
fn test_get_defines_dash_without_dash_use_no_dash_defines()
{
  let mut line = d3::Line::default();
  // Set a pattern but don't enable dashing
  line.dash_pattern_set( d3::DashPattern::V4( [ 0.1, 0.2, 0.3, 0.4 ] ) );

  let defines = line.defines_get();
  assert!( !defines.contains( "#define USE_DASH" ) );
}

#[ test ]
fn test_get_defines_dash_use_includes_default_pattern_v1()
{
  let mut line = d3::Line::default();
  line.dash_use( true );

  // Default DashPattern is V1, so enabling dash should include V1 define
  let defines = line.defines_get();
  assert!( defines.contains( "#define USE_DASH\n" ) );
  assert!( defines.contains( "#define USE_DASH_V1\n" ) );
}

// === DashPattern clone and copy ===

#[ test ]
fn test_dash_pattern_clone()
{
  let pattern = d3::DashPattern::V2( [ 0.3, 0.7 ] );
  let cloned = pattern.clone();
  match cloned
  {
    d3::DashPattern::V2( v ) => assert_eq!( v, [ 0.3, 0.7 ] ),
    _ => panic!( "Cloned pattern should be V2" ),
  }
}

#[ test ]
fn test_dash_pattern_copy()
{
  let pattern = d3::DashPattern::V4( [ 0.1, 0.2, 0.3, 0.4 ] );
  let copied = pattern;
  // pattern is still usable because DashPattern is Copy
  match ( pattern, copied )
  {
    ( d3::DashPattern::V4( a ), d3::DashPattern::V4( b ) ) => assert_eq!( a, b ),
    _ => panic!( "Both should be V4" ),
  }
}

#[ test ]
fn test_dash_offset_get_and_set()
{
  let mut line = d3::Line::default();
  assert_eq!( line.dash_offset_get(), 0.0 );

  line.dash_offset_set( 5.0 );
  assert_eq!( line.dash_offset_get(), 5.0 );
}

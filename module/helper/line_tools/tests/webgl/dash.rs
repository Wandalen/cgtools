
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

// === use_dash ===

#[ test ]
fn test_use_dash_enables_dash_define()
{
  let mut line = d3::Line::default();
  line.use_dash( true );

  let defines = line.get_defines();
  assert!( defines.contains( "#define USE_DASH\n" ) );
}

#[ test ]
fn test_use_dash_false_no_dash_define()
{
  let mut line = d3::Line::default();
  line.use_dash( false );

  let defines = line.get_defines();
  assert!( !defines.contains( "#define USE_DASH" ) );
}

#[ test ]
fn test_use_dash_toggle()
{
  let mut line = d3::Line::default();

  line.use_dash( true );
  assert!( line.get_defines().contains( "#define USE_DASH\n" ) );

  line.use_dash( false );
  assert!( !line.get_defines().contains( "#define USE_DASH" ) );
}

// === dash_pattern_set ===

#[ test ]
fn test_dash_pattern_set_v1()
{
  let mut line = d3::Line::default();
  line.use_dash( true );
  line.dash_pattern_set( d3::DashPattern::V1( 1.0 ) );

  let defines = line.get_defines();
  assert!( defines.contains( "#define USE_DASH_V1\n" ) );
  assert!( !defines.contains( "#define USE_DASH_V2" ) );
  assert!( !defines.contains( "#define USE_DASH_V3" ) );
  assert!( !defines.contains( "#define USE_DASH_V4" ) );
}

#[ test ]
fn test_dash_pattern_set_v2()
{
  let mut line = d3::Line::default();
  line.use_dash( true );
  line.dash_pattern_set( d3::DashPattern::V2( [ 0.5, 0.5 ] ) );

  let defines = line.get_defines();
  assert!( defines.contains( "#define USE_DASH_V2\n" ) );
  assert!( !defines.contains( "#define USE_DASH_V1" ) );
}

#[ test ]
fn test_dash_pattern_set_v3()
{
  let mut line = d3::Line::default();
  line.use_dash( true );
  line.dash_pattern_set( d3::DashPattern::V3( [ 0.5, 0.25, 0.25 ] ) );

  let defines = line.get_defines();
  assert!( defines.contains( "#define USE_DASH_V3\n" ) );
}

#[ test ]
fn test_dash_pattern_set_v4()
{
  let mut line = d3::Line::default();
  line.use_dash( true );
  line.dash_pattern_set( d3::DashPattern::V4( [ 0.4, 0.1, 0.4, 0.1 ] ) );

  let defines = line.get_defines();
  assert!( defines.contains( "#define USE_DASH_V4\n" ) );
}

#[ test ]
fn test_dash_pattern_set_same_variant_no_defines_change()
{
  let mut line = d3::Line::default();
  line.use_dash( true );

  // Default is V1, setting V1 again with different value
  // should not change the defines output since discriminant is the same
  line.dash_pattern_set( d3::DashPattern::V1( 0.5 ) );
  let defines1 = line.get_defines();

  line.dash_pattern_set( d3::DashPattern::V1( 0.8 ) );
  let defines2 = line.get_defines();

  assert_eq!( defines1, defines2 );
}

#[ test ]
fn test_dash_pattern_set_different_variant_changes_defines()
{
  let mut line = d3::Line::default();
  line.use_dash( true );
  line.dash_pattern_set( d3::DashPattern::V1( 0.5 ) );

  let defines1 = line.get_defines();
  assert!( defines1.contains( "#define USE_DASH_V1\n" ) );

  line.dash_pattern_set( d3::DashPattern::V2( [ 0.3, 0.7 ] ) );

  let defines2 = line.get_defines();
  assert!( defines2.contains( "#define USE_DASH_V2\n" ) );
  assert!( !defines2.contains( "#define USE_DASH_V1" ) );
}

// === get_defines combinations ===

#[ test ]
fn test_get_defines_empty_by_default()
{
  let line = d3::Line::default();
  let defines = line.get_defines();
  assert_eq!( defines, "" );
}

#[ test ]
fn test_get_defines_vertex_color()
{
  let mut line = d3::Line::default();
  line.use_vertex_color( true );

  let defines = line.get_defines();
  assert!( defines.contains( "#define USE_VERTEX_COLORS\n" ) );
}

#[ test ]
fn test_get_defines_alpha_to_coverage()
{
  let mut line = d3::Line::default();
  line.use_alpha_to_coverage( true );

  let defines = line.get_defines();
  assert!( defines.contains( "#define USE_ALPHA_TO_COVERAGE\n" ) );
}

#[ test ]
fn test_get_defines_world_units()
{
  let mut line = d3::Line::default();
  line.use_world_units( true );

  let defines = line.get_defines();
  assert!( defines.contains( "#define USE_WORLD_UNITS\n" ) );
}

#[ test ]
fn test_get_defines_all_flags()
{
  let mut line = d3::Line::default();
  line.use_vertex_color( true );
  line.use_alpha_to_coverage( true );
  line.use_world_units( true );
  line.use_dash( true );
  line.dash_pattern_set( d3::DashPattern::V3( [ 0.5, 0.25, 0.25 ] ) );

  let defines = line.get_defines();
  assert!( defines.contains( "#define USE_VERTEX_COLORS\n" ) );
  assert!( defines.contains( "#define USE_ALPHA_TO_COVERAGE\n" ) );
  assert!( defines.contains( "#define USE_WORLD_UNITS\n" ) );
  assert!( defines.contains( "#define USE_DASH\n" ) );
  assert!( defines.contains( "#define USE_DASH_V3\n" ) );
}

#[ test ]
fn test_get_defines_dash_without_use_dash_no_dash_defines()
{
  let mut line = d3::Line::default();
  // Set a pattern but don't enable dashing
  line.dash_pattern_set( d3::DashPattern::V4( [ 0.1, 0.2, 0.3, 0.4 ] ) );

  let defines = line.get_defines();
  assert!( !defines.contains( "#define USE_DASH" ) );
}

#[ test ]
fn test_get_defines_use_dash_includes_default_pattern_v1()
{
  let mut line = d3::Line::default();
  line.use_dash( true );

  // Default DashPattern is V1, so enabling dash should include V1 define
  let defines = line.get_defines();
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

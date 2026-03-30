#![ allow( clippy::min_ident_chars ) ]
#![ allow( clippy::float_cmp ) ]

//! Types tests

use tilemap_renderer::types::*;

#[ test ]
fn resource_id_type_safety()
{
  let a : ResourceId< asset::Image > = ResourceId::new( 5 );
  let b : ResourceId< asset::Image > = ResourceId::new( 5 );
  let c : ResourceId< asset::Image > = ResourceId::new( 7 );
  assert_eq!( a, b );
  assert_ne!( a, c );
  assert_eq!( a.inner(), 5 );
}

#[ test ]
fn resource_id_debug()
{
  let id : ResourceId< asset::Sprite > = ResourceId::new( 42 );
  assert_eq!( format!( "{id:?}" ), "ResourceId(42)" );
}

#[ test ]
fn transform_default_is_identity()
{
  let t = Transform::default();
  assert_eq!( t.position, [ 0.0, 0.0 ] );
  assert_eq!( t.rotation, 0.0 );
  assert_eq!( t.scale, [ 1.0, 1.0 ] );
  assert_eq!( t.skew, [ 0.0, 0.0 ] );
  assert_eq!( t.depth, 0.0 );
}

#[ test ]
fn to_mat3_identity()
{
  let t = Transform::default();
  let m = t.to_mat3();
  let expected =
  [
    1.0, 0.0, 0.0,
    0.0, 1.0, 0.0,
    0.0, 0.0, 1.0,
  ];
  for ( a, b ) in m.iter().zip( expected.iter() )
  {
    assert!( ( a - b ).abs() < 1e-6, "expected {b} got {a}" );
  }
}

#[ test ]
fn to_mat3_translation()
{
  let t = Transform { position : [ 10.0, 20.0 ], ..Default::default() };
  let m = t.to_mat3();
  assert!( ( m[ 6 ] - 10.0 ).abs() < 1e-6 );
  assert!( ( m[ 7 ] - 20.0 ).abs() < 1e-6 );
  assert!( ( m[ 0 ] - 1.0 ).abs() < 1e-6 );
  assert!( ( m[ 4 ] - 1.0 ).abs() < 1e-6 );
}

#[ test ]
fn to_mat3_scale()
{
  let t = Transform { scale : [ 2.0, 3.0 ], ..Default::default() };
  let m = t.to_mat3();
  assert!( ( m[ 0 ] - 2.0 ).abs() < 1e-6 );
  assert!( ( m[ 4 ] - 3.0 ).abs() < 1e-6 );
  assert!( ( m[ 1 ] ).abs() < 1e-6 );
  assert!( ( m[ 3 ] ).abs() < 1e-6 );
}

#[ test ]
fn to_mat3_rotation_90()
{
  let t = Transform { rotation : core::f32::consts::FRAC_PI_2, ..Default::default() };
  let m = t.to_mat3();
  assert!( m[ 0 ].abs() < 1e-6, "m00={}", m[ 0 ] );
  assert!( ( m[ 1 ] - 1.0 ).abs() < 1e-6, "m10={}", m[ 1 ] );
  assert!( ( m[ 3 ] + 1.0 ).abs() < 1e-6, "m01={}", m[ 3 ] );
  assert!( m[ 4 ].abs() < 1e-6, "m11={}", m[ 4 ] );
}

#[ test ]
fn render_config_default()
{
  let c = RenderConfig::default();
  assert_eq!( c.width, 800 );
  assert_eq!( c.height, 600 );
  assert_eq!( c.antialias, Antialias::Default );
  assert_eq!( c.background, [ 0.0, 0.0, 0.0, 1.0 ] );
}

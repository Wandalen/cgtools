//! Types tests

use tilemap_renderer::types::*;

#[ test ]
fn resource_id_type_safety()
{
  let id_a : ResourceId< asset::Image > = ResourceId::new( 5 );
  let id_b : ResourceId< asset::Image > = ResourceId::new( 5 );
  let id_c : ResourceId< asset::Image > = ResourceId::new( 7 );
  assert_eq!( id_a, id_b );
  assert_ne!( id_a, id_c );
  assert_eq!( id_a.inner(), 5 );
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
  let transform = Transform::default();
  assert!( transform.position[ 0 ].abs() < 1e-6 );
  assert!( transform.position[ 1 ].abs() < 1e-6 );
  assert!( transform.rotation.abs() < 1e-6 );
  assert!( ( transform.scale[ 0 ] - 1.0 ).abs() < 1e-6 );
  assert!( ( transform.scale[ 1 ] - 1.0 ).abs() < 1e-6 );
  assert!( transform.skew[ 0 ].abs() < 1e-6 );
  assert!( transform.skew[ 1 ].abs() < 1e-6 );
  assert!( transform.depth.abs() < 1e-6 );
}

#[ test ]
fn to_mat3_identity()
{
  let transform = Transform::default();
  let mat = transform.to_mat3();
  let expected =
  [
    1.0, 0.0, 0.0,
    0.0, 1.0, 0.0,
    0.0, 0.0, 1.0,
  ];
  for ( actual, expected_val ) in mat.iter().zip( expected.iter() )
  {
    assert!( ( actual - expected_val ).abs() < 1e-6, "expected {expected_val} got {actual}" );
  }
}

#[ test ]
fn to_mat3_translation()
{
  let transform = Transform { position : [ 10.0, 20.0 ], ..Default::default() };
  let mat = transform.to_mat3();
  assert!( ( mat[ 6 ] - 10.0 ).abs() < 1e-6 );
  assert!( ( mat[ 7 ] - 20.0 ).abs() < 1e-6 );
  assert!( ( mat[ 0 ] - 1.0 ).abs() < 1e-6 );
  assert!( ( mat[ 4 ] - 1.0 ).abs() < 1e-6 );
}

#[ test ]
fn to_mat3_scale()
{
  let transform = Transform { scale : [ 2.0, 3.0 ], ..Default::default() };
  let mat = transform.to_mat3();
  assert!( ( mat[ 0 ] - 2.0 ).abs() < 1e-6 );
  assert!( ( mat[ 4 ] - 3.0 ).abs() < 1e-6 );
  assert!( ( mat[ 1 ] ).abs() < 1e-6 );
  assert!( ( mat[ 3 ] ).abs() < 1e-6 );
}

#[ test ]
fn to_mat3_rotation_90()
{
  let transform = Transform { rotation : core::f32::consts::FRAC_PI_2, ..Default::default() };
  let mat = transform.to_mat3();
  assert!( mat[ 0 ].abs() < 1e-6, "m00={}", mat[ 0 ] );
  assert!( ( mat[ 1 ] - 1.0 ).abs() < 1e-6, "m10={}", mat[ 1 ] );
  assert!( ( mat[ 3 ] + 1.0 ).abs() < 1e-6, "m01={}", mat[ 3 ] );
  assert!( mat[ 4 ].abs() < 1e-6, "m11={}", mat[ 4 ] );
}

#[ test ]
fn render_config_default()
{
  let config = RenderConfig::default();
  assert_eq!( config.width, 800 );
  assert_eq!( config.height, 600 );
  assert_eq!( config.antialias, Antialias::Default );
  assert!( config.background[ 0 ].abs() < 1e-6 );
  assert!( config.background[ 1 ].abs() < 1e-6 );
  assert!( config.background[ 2 ].abs() < 1e-6 );
  assert!( ( config.background[ 3 ] - 1.0 ).abs() < 1e-6 );
}

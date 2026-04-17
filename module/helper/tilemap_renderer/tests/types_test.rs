#![ allow( clippy::min_ident_chars ) ] // Short names like x, y, m are idiomatic in math/graphics contexts throughout this crate

//! Types tests.
//!
//! Covers:
//! - `Transform` identity state, translation matrix slots, scale diagonal, 90-degree rotation
//! - `ResourceId` type-safe equality and debug formatting
//! - `RenderConfig` default field values (width, height, antialias, background color)

use tilemap_renderer::types::*;

/// Verifies that two `ResourceId<Image>` values with the same inner id compare
/// equal, and that a different id compares unequal — type-safe id equality.
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

/// Verifies that `ResourceId` formats as `"ResourceId(N)"` so that
/// debug output is readable in test failure messages.
#[ test ]
fn resource_id_debug()
{
  let id : ResourceId< asset::Sprite > = ResourceId::new( 42 );
  assert_eq!( format!( "{id:?}" ), "ResourceId(42)" );
}

/// Verifies that `Transform::default()` is the identity transform:
/// zero position, zero rotation, unit scale, zero skew, zero depth.
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

/// Verifies that the identity `Transform` produces a 3×3 identity matrix
/// from `to_mat3()`, element-by-element within float tolerance.
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

/// Verifies that a translation-only `Transform` places the translation
/// vector in the correct matrix slots (column-major: indices 6 and 7).
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

/// Verifies that a scale-only `Transform` places the scale factors on
/// the matrix diagonal and zeroes the off-diagonal elements.
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

/// Verifies that a 90-degree rotation produces the expected sine/cosine
/// values in the rotation slots of the matrix (indices 0, 1, 3, 4).
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

/// Verifies the Y-up CCW invariant: a 90° CCW rotation maps world direction (1, 0) to (0, 1).
/// In column-major layout this means m[1] = +sin = +1 and m[3] = -sin = -1.
/// This is the contract every adapter must preserve.
#[ test ]
fn to_mat3_ccw_positive_rotation()
{
  let t = Transform { rotation : core::f32::consts::FRAC_PI_2, ..Default::default() };
  let m = t.to_mat3();
  assert!( ( m[ 1 ] - 1.0 ).abs() < 1e-6, "sin(π/2) should be +1 for CCW, got {}", m[ 1 ] );
  assert!( ( m[ 3 ] + 1.0 ).abs() < 1e-6, "-sin(π/2) should be -1 for CCW, got {}", m[ 3 ] );
}

/// Verifies that `RenderConfig::default()` produces the expected
/// width (800), height (600), antialias mode, and background color.
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

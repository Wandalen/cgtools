//! Verifies the glTF loader's `KHR_lights_punctual` extraction
//! ( `renderer::webgl::loaders::gltf::light_list_get` ) — the pure
//! extension-parsing logic that turns an already-parsed `&gltf::Gltf`
//! document into `Light` domain values through pattern-matching and field
//! mapping only, with zero `WebGl2RenderingContext`/`gl::` calls anywhere in
//! its body. Mirrors `gltf_loader_tests.rs`'s `asset_uri_resolve`
//! promotion-and-export precedent for a second pure sub-surface of the same
//! loader.

use renderer::webgl::loaders::gltf::{ light_list_get, light_get };
use renderer::webgl::{ Light, PointLight, DirectLight, SpotLight, Node };
use mingl::{ F32x3, math };
use rustc_hash::FxHashMap;
use approx::assert_abs_diff_eq;

const MIXED_FIXTURE : &str = r#"
{
  "asset": { "version": "2.0" },
  "extensions":
  {
    "KHR_lights_punctual":
    {
      "lights":
      [
        { "type": "point", "color": [ 1.0, 0.0, 0.0 ], "intensity": 500.0, "range": 10.0 },
        { "type": "directional", "color": [ 0.0, 1.0, 0.0 ], "intensity": 2.5 },
        { "type": "spot", "color": [ 0.0, 0.0, 1.0 ], "intensity": 800.0, "spot": { "innerConeAngle": 0.1, "outerConeAngle": 0.5 } }
      ]
    }
  }
}
"#;

#[ test ]
fn mixed_fixture_yields_three_lights_with_correct_fields()
{
  let gltf = gltf::Gltf::from_slice( MIXED_FIXTURE.as_bytes() ).unwrap();
  let lights = light_list_get( &gltf ).expect( "extension present, must be Some" );
  assert_eq!( lights.len(), 3 );

  match lights.get( &0 ).expect( "index 0 present" )
  {
    Light::Point( PointLight { color, strength, range, .. } ) =>
    {
      assert_eq!( *color, F32x3::from_slice( &[ 1.0, 0.0, 0.0 ] ) );
      assert_abs_diff_eq!( *strength, 500.0 );
      assert_abs_diff_eq!( *range, 10.0 );
    },
    other => panic!( "index 0 expected Light::Point, got {other:?}" )
  }

  match lights.get( &1 ).expect( "index 1 present" )
  {
    Light::Direct( DirectLight { color, strength, .. } ) =>
    {
      assert_eq!( *color, F32x3::from_slice( &[ 0.0, 1.0, 0.0 ] ) );
      assert_abs_diff_eq!( *strength, 2.5 );
    },
    other => panic!( "index 1 expected Light::Direct, got {other:?}" )
  }

  match lights.get( &2 ).expect( "index 2 present" )
  {
    Light::Spot( SpotLight { color, strength, range, inner_cone_angle, outer_cone_angle, .. } ) =>
    {
      assert_eq!( *color, F32x3::from_slice( &[ 0.0, 0.0, 1.0 ] ) );
      assert_abs_diff_eq!( *strength, 800.0 );
      assert_abs_diff_eq!( *range, 10.0 );
      assert_abs_diff_eq!( *inner_cone_angle, 0.1 );
      assert_abs_diff_eq!( *outer_cone_angle, 0.5 );
    },
    other => panic!( "index 2 expected Light::Spot, got {other:?}" )
  }
}

const POINT_MISSING_RANGE_FIXTURE : &str = r#"
{
  "asset": { "version": "2.0" },
  "extensions":
  {
    "KHR_lights_punctual":
    {
      "lights":
      [
        { "type": "point", "color": [ 1.0, 1.0, 1.0 ], "intensity": 1.0 }
      ]
    }
  }
}
"#;

#[ test ]
fn point_light_missing_range_is_silently_skipped()
{
  let gltf = gltf::Gltf::from_slice( POINT_MISSING_RANGE_FIXTURE.as_bytes() ).unwrap();
  let lights = light_list_get( &gltf ).expect( "extension present, must be Some" );
  assert_eq!( lights.len(), 0 );
}

const EMPTY_LIGHTS_ARRAY_FIXTURE : &str = r#"
{
  "asset": { "version": "2.0" },
  "extensions":
  {
    "KHR_lights_punctual":
    {
      "lights": []
    }
  }
}
"#;

#[ test ]
fn empty_lights_array_yields_some_empty_map()
{
  let gltf = gltf::Gltf::from_slice( EMPTY_LIGHTS_ARRAY_FIXTURE.as_bytes() ).unwrap();
  let lights = light_list_get( &gltf ).expect( "extension present, must be Some" );
  assert_eq!( lights.len(), 0 );
}

const NO_EXTENSION_FIXTURE : &str = r#"
{
  "asset": { "version": "2.0" }
}
"#;

#[ test ]
fn missing_extension_key_yields_none()
{
  let gltf = gltf::Gltf::from_slice( NO_EXTENSION_FIXTURE.as_bytes() ).unwrap();
  assert!( light_list_get( &gltf ).is_none() );
}

const NODE_WITH_LIGHT_FIXTURE : &str = r#"
{
  "asset": { "version": "2.0" },
  "extensions":
  {
    "KHR_lights_punctual":
    {
      "lights": [ { "type": "directional", "color": [ 1.0, 1.0, 1.0 ], "intensity": 1.0 } ]
    }
  },
  "nodes":
  [
    { "extensions": { "KHR_lights_punctual": { "light": 0 } } }
  ]
}
"#;

#[ test ]
fn light_get_resolves_node_level_light_reference()
{
  // BUG-189: `light_get` read the node-level `KHR_lights_punctual` reference via
  // `gltf_node.extensions()`, the catch-all for extension data *unknown* to this crate --
  // but `KHR_lights_punctual` is a named, typed field this crate's `gltf-json` deserializes
  // separately, so it never appeared there. `light_get` always returned `None`, for every
  // node, in every glTF asset -- a distinct defect from BUG-172's direction-formula bug in
  // the same function, and one that made BUG-172's fix unreachable dead code until fixed.
  let gltf = gltf::Gltf::from_slice( NODE_WITH_LIGHT_FIXTURE.as_bytes() ).unwrap();
  let gltf_node = gltf.nodes().next().expect( "fixture declares one node" );

  let mut node = Node::new();
  node.translation_set( [ 3.0, 4.0, 5.0 ] );

  let mut lights : FxHashMap< usize, Light > = FxHashMap::default();
  lights.insert
  (
    0,
    Light::Point( PointLight { position : F32x3::from_array( [ 0.0, 0.0, 0.0 ] ), color : F32x3::from_array( [ 1.0, 1.0, 1.0 ] ), strength : 1.0, range : 10.0 } )
  );

  match light_get( &gltf_node, &node, &lights ).expect( "node references a valid light index, must resolve to Some" )
  {
    Light::Point( point ) => assert_abs_diff_eq!( point.position, node.translation_get() ),
    other => panic!( "expected Light::Point, got {other:?}" )
  }
}

#[ test ]
fn light_get_derives_direction_from_rotation_not_translation()
{
  // BUG-172: `light_get` used to derive `Direct`/`Spot` light direction from the node's
  // *translation* (a world position, not a facing direction), only falling back to the
  // correct rotation-based formula when translation magnitude was within 1cm of the origin.
  // A large, non-origin translation plus a recognizable non-identity rotation distinguishes
  // the two: pre-fix, `direction` equaled the raw translation vector; post-fix, it's the
  // rotation-derived unit vector -- trivially distinguishable by both value and magnitude.
  let gltf = gltf::Gltf::from_slice( NODE_WITH_LIGHT_FIXTURE.as_bytes() ).unwrap();
  let gltf_node = gltf.nodes().next().expect( "fixture declares one node" );

  let mut node = Node::new();
  node.translation_set( [ 10.0, 20.0, 30.0 ] );
  let rotation = math::QuatF32::from_angle_y( 90f32.to_radians() );
  node.rotation_set( rotation );

  let forward = F32x3::from_array( [ 0.0, 0.0, -1.0 ] );
  let rot_matrix = math::d2::F32x3x3::from_quat( rotation );
  let expected_direction = ( rot_matrix * forward ).normalize();
  let old_buggy_direction = node.translation_get();
  assert_ne!( expected_direction, old_buggy_direction, "fixture must discriminate old buggy output from the fix" );

  let mut direct_lights : FxHashMap< usize, Light > = FxHashMap::default();
  direct_lights.insert
  (
    0,
    Light::Direct( DirectLight { direction : F32x3::from_array( [ 0.0, 0.0, 0.0 ] ), color : F32x3::from_array( [ 1.0, 1.0, 1.0 ] ), strength : 1.0 } )
  );

  match light_get( &gltf_node, &node, &direct_lights ).expect( "extension present and light index valid, must be Some" )
  {
    Light::Direct( direct ) => assert_abs_diff_eq!( direct.direction, expected_direction ),
    other => panic!( "expected Light::Direct, got {other:?}" )
  }

  let mut spot_lights : FxHashMap< usize, Light > = FxHashMap::default();
  spot_lights.insert
  (
    0,
    Light::Spot
    (
      SpotLight
      {
        position : F32x3::from_array( [ 0.0, 0.0, 0.0 ] ),
        direction : F32x3::from_array( [ 0.0, 0.0, 0.0 ] ),
        color : F32x3::from_array( [ 1.0, 1.0, 1.0 ] ),
        strength : 1.0,
        range : 10.0,
        inner_cone_angle : 0.1,
        outer_cone_angle : 0.5,
        use_light_map : false,
      }
    )
  );

  match light_get( &gltf_node, &node, &spot_lights ).expect( "extension present and light index valid, must be Some" )
  {
    Light::Spot( spot ) =>
    {
      assert_abs_diff_eq!( spot.direction, expected_direction );
      assert_abs_diff_eq!( spot.position, node.translation_get() );
    },
    other => panic!( "expected Light::Spot, got {other:?}" )
  }
}

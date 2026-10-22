//! The glTF loader's `KHR_lights_punctual` lights : the document's light list, and the light a
//! node references, placed by that node.

mod private
{
  use minwebgl as gl;
  use mingl::F32x3;
  use rustc_hash::FxHashMap;
  use crate::webgl::{ DirectLight, Light, Node, PointLight, SpotLight };

  /// Extracts all `KHR_lights_punctual` lights from a parsed glTF document, keyed by their light index.
  #[ must_use ]
  pub fn light_list_get( gltf : &gltf::Gltf ) -> Option< FxHashMap< usize, Light > >
  {
    let mut lights = FxHashMap::default();
    for ( i, gltf_light ) in gltf.lights()?.enumerate()
    {
      let light_type = gltf_light.kind();
      let light =  match light_type
      {
        gltf::khr_lights_punctual::Kind::Point =>
        {
          let Some( range ) = gltf_light.range()
          else
          {
            continue;
          };
          Light::Point
          (
            PointLight
            {
              position : F32x3::default(),
              color : F32x3::from_slice( &gltf_light.color() ),
              strength : gltf_light.intensity(),
              range
            }
          )
        },
        gltf::khr_lights_punctual::Kind::Directional =>
        {
          Light::Direct
          (
            DirectLight
            {
              direction : F32x3::default(),
              color : F32x3::from_slice( &gltf_light.color() ),
              strength : gltf_light.intensity(),
            }
          )
        },
        gltf::khr_lights_punctual::Kind::Spot { inner_cone_angle, outer_cone_angle } =>
        {
          let color = gltf_light.color();
          let strength = gltf_light.intensity();
          let range = gltf_light.range().unwrap_or( 10.0 );

          Light::Spot
          (
            SpotLight
            {
              position : F32x3::default(),
              direction : F32x3::default(),
              color: color.into(),
              strength,
              range,
              inner_cone_angle,
              outer_cone_angle,
              use_light_map : false,
            }
          )
        }
      };

      lights.insert( i, light );
    }

    Some( lights )
  }

  /// Resolves the [`Light`] a glTF node's `KHR_lights_punctual` extension references
  /// (looked up by index in `lights`, e.g. from [`light_list_get`]), positioned/oriented
  /// using `node`'s own resolved translation/rotation.
  #[ must_use ]
  pub fn light_get< S : std::hash::BuildHasher >( gltf_node : &gltf::Node< '_ >, node : &Node, lights : &std::collections::HashMap< usize, Light, S > ) -> Option< Light >
  {
    // Fix(BUG-189): the node-level `KHR_lights_punctual` reference was read via
    // `gltf_node.extensions()`, the catch-all for extension data *unknown* to this crate
    // version -- but `KHR_lights_punctual` is a named, typed field this crate's `gltf-json`
    // deserializes separately (`#[serde(rename = "KHR_lights_punctual")]`), so `#[serde(flatten)]`
    // never leaves it in that catch-all. `get_key_value` always returned `None`, so this
    // function never resolved a single node-level light reference, for any glTF asset.
    // Root cause: this crate exposes a dedicated typed accessor, `gltf::Node::light()`, for
    // exactly this extension (gated by the same `KHR_lights_punctual` Cargo feature this crate
    // already enables) -- `light_list_get` just above already uses the equivalent
    // document-level typed accessor (`gltf.lights()`), only this per-node lookup didn't.
    // Pitfall: a crate offering both a generic "unknown extensions" catch-all and a typed
    // accessor for a *specific* known extension makes the catch-all silently exclude that
    // extension the moment its typed-support feature is enabled -- reaching for the generic
    // path out of habit, instead of the type's own dedicated accessor, fails silently (`None`),
    // never a compile error.
    let light_id = gltf_node.light()?.index();

    lights.get( &light_id ).copied()
    .map
    (
      | light |
      {
        match light
        {
          Light::Point( mut point_light ) =>
          {
            point_light.position = node.translation_get();
            Light::Point( point_light )
          },
          // Fix(BUG-172): both arms used to derive `direction` from `node.translation_get()` --
          // a light's world position, not its facing direction. `Direct` only fell back to the
          // (correct) rotation-based formula when the raw translation's magnitude was below
          // `DIRECTION_LIGHT_MIN_MAGNITUDE`, i.e. only for a light sitting within 1cm of the
          // world origin; `Spot` had no rotation-based fallback at all.
          // Root cause: per glTF's `KHR_lights_punctual`, facing direction comes exclusively
          // from the node's rotation (local -Z axis) -- never its translation. Both arms now
          // compute it unconditionally the same way.
          // Pitfall: a magnitude-gated "fallback" that's actually the only physically correct
          // formula silently masks the bug for every test fixture that happens to sit near the
          // origin, while still being wrong for every other placement.
          Light::Direct( mut direct_light ) =>
          {
            let forward = gl::F32x3::from_array( [ 0.0, 0.0, -1.0 ] );
            let rot_matrix = gl::math::d2::F32x3x3::from_quat( node.rotation_get() );
            direct_light.direction = ( rot_matrix * forward ).normalize();
            Light::Direct( direct_light )
          },
          Light::Spot( mut spot_light ) =>
          {
            let forward = gl::F32x3::from_array( [ 0.0, 0.0, -1.0 ] );
            let rot_matrix = gl::math::d2::F32x3x3::from_quat( node.rotation_get() );
            spot_light.position = node.translation_get();
            spot_light.direction = ( rot_matrix * forward ).normalize();
            Light::Spot( spot_light )
          }
        }
      }
    )
  }
}

crate::mod_interface!
{
  own use
  {
    light_list_get,
    light_get,
  };
}

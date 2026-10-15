//! Set of tools for generating 3D geometry
#![ doc( html_root_url = "https://docs.rs/primitive_generation/latest/primitive_generation/" ) ]
#![ cfg_attr( doc, doc = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/", "readme.md" ) ) ) ]
#![ cfg_attr( not( doc ), doc = "Set of tools for generating 3D geometry" ) ]

mod private
{

}

::mod_interface::mod_interface!
{
  own use ::mod_interface::mod_interface;

  /// Text generation and font processing utilities.
  layer text;

  /// Basic geometric primitive creation.
  layer primitive;

  /// Data structures for primitive attributes and transformations.
  layer primitive_data;

  /// Procedural solid primitives (box/cylinder/cone/torus/icosphere) as raw
  /// `( positions, indices )` pairs, independent of the `PrimitiveData`/GLTF
  /// pipeline above.
  layer solid;

  /// Catmull-Rom spline evaluation over a waypoint path.
  layer spline;
}
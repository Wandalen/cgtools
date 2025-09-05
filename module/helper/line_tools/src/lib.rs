//! Line drawing and manipulation utilities for 2D and 3D graphics.
#![ doc( html_root_url = "https://docs.rs/line_tools/latest/line_tools/" ) ]
#![ cfg_attr( doc, doc = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/", "readme.md" ) ) ) ]
#![ cfg_attr( not( doc ), doc = "Line drawing and manipulation utilities for 2D and 3D graphics" ) ]

mod private
{
  /// Provides method to get mesh geometry from the structure
  pub trait Geometry
  {
    /// Generates the geometry.
    ///
    /// This method returns a tuple containing the vertices, indices, uvs, and the number of
    /// elements for the mesh.
    fn geometry( &self ) -> ( Vec< f32 >, Vec< u32 >, Vec< f32 >, usize );
  }
}

::mod_interface::mod_interface!
{
  own use ::mod_interface::mod_interface;
  own use
  {
    Geometry
  };

  /// A layer for 2D graphics-related functionalities.
  layer d2;
  /// A layer for 3D graphics-related functionalities.
  layer d3;

  /// A layer for mesh generation and manipulation.
  layer mesh;
  /// A layer for shader programs and related functionality.
  layer program;

  /// A layer for helper functions and utilities used by other modules.
  layer helpers;
}
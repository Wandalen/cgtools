//! Geometry creation functions

use minwebgl as gl;
use gl::{ GL, BufferDescriptor, WebglError, geometry::BoundingBox };
use renderer::webgl::{ loaders::gltf, AttributeInfo, IndexInfo };
use crate::types::RenderGeometry;

/// Setup geometry for light volumes and visualization spheres
pub fn create_geometry
(
  gl : &GL,
  sphere : &gltf::GLTF,
  translation_buffer : &gl::WebGlBuffer,
  radius_buffer : &gl::WebGlBuffer,
) -> Result< RenderGeometry, gl::WebglError >
{
  let mut light_volume = create_light_volume( gl )?;

  let translation_attribute = AttributeInfo
  {
    slot : 1,
    buffer : translation_buffer.clone(),
    descriptor : BufferDescriptor::new::< [ f32; 3 ] >().divisor( 1 ),
    bounding_box : BoundingBox::default(),
  };
  light_volume.add_attribute( gl, "a_translation", translation_attribute )?;

  let radius_attribute = AttributeInfo
  {
    slot : 2,
    buffer : radius_buffer.clone(),
    descriptor : BufferDescriptor::new::< f32 >().divisor( 1 ),
    bounding_box : BoundingBox::default(),
  };
  light_volume.add_attribute( gl, "a_radius", radius_attribute )?;

  // Setup sphere geometry
  let sphere_mesh = &sphere.meshes[ 0 ];
  let sphere_primitive = &sphere_mesh.borrow().primitives[ 0 ];
  let light_sphere = sphere_primitive.borrow().geometry.clone();

  let sphere_translation_attribute = AttributeInfo
  {
    slot : 1,
    buffer : translation_buffer.clone(),
    descriptor : BufferDescriptor::new::< [ f32; 3 ] >().divisor( 1 ),
    bounding_box : BoundingBox::default(),
  };
  light_sphere.borrow_mut().add_attribute( gl, "a_translation", sphere_translation_attribute )?;

  Ok( RenderGeometry { light_volume, light_sphere } )
}

/// Creates the geometry for a light volume (a unit cube).
pub fn create_light_volume( gl : &GL ) -> Result< renderer::webgl::Geometry, WebglError >
{
  // Define cube vertices
  static CUBE_VERTICES : &[ f32 ] =
  &[
    // Front face
    -1.0, -1.0,  1.0,
     1.0, -1.0,  1.0,
     1.0,  1.0,  1.0,
    -1.0,  1.0,  1.0,
    // Back face
    -1.0, -1.0, -1.0,
     1.0, -1.0, -1.0,
     1.0,  1.0, -1.0,
    -1.0,  1.0, -1.0,
  ];

  // Define cube indices
  static CUBE_INDICES : &[ u32 ] =
  &[
    // Front face
    0, 1, 2, 0, 2, 3,
    // Back face
    4, 6, 5, 4, 7, 6,
    // Top face
    3, 2, 6, 3, 6, 7,
    // Bottom face
    0, 5, 1, 0, 4, 5,
    // Right face
    1, 5, 6, 1, 6, 2,
    // Left face
    0, 3, 7, 0, 7, 4,
  ];

  // Unbind any currently bound vertex array
  gl.bind_vertex_array( None );

  // Create a new Geometry object
  let mut light_volume = renderer::webgl::Geometry::new( &gl )?;

  // Create and upload the position buffer
  let position_buffer = gl::buffer::create( gl )?;
  gl::buffer::upload( gl, &position_buffer, CUBE_VERTICES, GL::STATIC_DRAW );
  // Add the position attribute to the geometry
  let attribute = AttributeInfo
  {
    slot : 0, // Attribute slot 0
    buffer : position_buffer,
    descriptor : BufferDescriptor::new::< [ f32; 3 ] >(), // Non-instanced attribute
    bounding_box : BoundingBox::default(),
  };
  light_volume.add_attribute( &gl, "position", attribute )?;

  // Create and upload the index buffer
  let index_buffer = gl::buffer::create( gl )?;
  gl::index::upload( gl, &index_buffer, CUBE_INDICES, GL::STATIC_DRAW );
  // Add the index buffer to the geometry
  let index = IndexInfo
  {
    buffer : index_buffer,
    count : CUBE_INDICES.len() as u32,
    offset : 0,
    data_type : GL::UNSIGNED_INT,
  };
  light_volume.add_index( &gl, index )?;

  Ok( light_volume )
}

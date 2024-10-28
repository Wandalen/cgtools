use std::
{
  collections::HashMap, 
  sync::{ Arc, Mutex }
};

use minwebgl as gl;
use gl::GL;

use crate::material::GLMaterial;

pub struct GLMesh
{
  material : GLMaterial,
  vao : gl::WebGlVertexArrayObject,
  indices_amount : i32
}

impl GLMesh 
{
  pub fn material( &self ) -> &GLMaterial
  {
    &self.material
  }

  pub fn from_tobj_model
  ( 
    gl : &GL, 
    model : &tobj::Model, 
    materials : &[ GLMaterial ] 
  ) 
  -> Result< Self, gl::WebglError >
  {
    // Get the material or generate a standart one
    let material = 
    match model.mesh.material_id 
    {
      Some( id ) if id < materials.len() => 
      { 
        materials[ id ].clone() 
      }
      _ => 
      {
        let mtl = GLMaterial::new_simple( gl )?;
        mtl
      }
    };

    let position_buffer =  gl::buffer::create( &gl )?;
    let normal_buffer = gl::buffer::create( &gl )?;
    let uv_buffer = gl::buffer::create( &gl )?;

    gl::buffer::upload( &gl, &position_buffer, &model.mesh.positions, GL::STATIC_DRAW );
    gl::buffer::upload( &gl, &normal_buffer, &model.mesh.normals, GL::STATIC_DRAW );
    gl::buffer::upload( &gl, &uv_buffer, &model.mesh.texcoords, GL::STATIC_DRAW );

    let vao = gl::vao::create( &gl )?;
    gl.bind_vertex_array( Some( &vao ) );
    gl::BufferDescriptor::new::< [ f32; 3 ] >().stride( 3 ).offset( 0 ).attribute_pointer( &gl, 0, &position_buffer )?;
    gl::BufferDescriptor::new::< [ f32; 3 ] >().stride( 3 ).offset( 0 ).attribute_pointer( &gl, 1, &normal_buffer )?;
    gl::BufferDescriptor::new::< [ f32; 2 ] >().stride( 2 ).offset( 0 ).attribute_pointer( &gl, 2, &uv_buffer )?;

    let index_buffer = gl::buffer::create( &gl )?;
    gl::index::upload( &gl, &index_buffer, &model.mesh.indices, GL::STATIC_DRAW );

    let indices_amount = model.mesh.indices.len() as i32;

    let mesh_gl = GLMesh
    {
      material,
      vao,
      indices_amount
    };

    Ok( mesh_gl )
  }

  pub fn set_perpsective( &self, gl : &GL, perspective_matrix : &glam::Mat4 )
  {
    gl.use_program( Some( &self.material.program ) );

    gl::uniform::matrix_upload
    ( 
      &gl, 
      gl.get_uniform_location( &self.material.program, "projectionMatrix" ), 
      &perspective_matrix.to_cols_array()[ .. ], 
      true 
    ).unwrap();
  }

  pub fn update
  ( 
    &self, 
    gl : &GL, 
    camera_position : &[ f32; 3 ], 
    view_matrix : &[ f32; 16 ] 
  )
  {
    gl.use_program( Some( &self.material.program ) );

    gl::uniform::upload
    (
      &gl, 
      gl.get_uniform_location( &self.material.program, "cameraPosition" ), 
      &camera_position[ .. ]
    ).unwrap();

    gl::uniform::matrix_upload
    ( 
      &gl, 
      gl.get_uniform_location( &self.material.program, "viewMatrix" ), 
      &view_matrix[ .. ], 
      true 
    ).unwrap();
  }
  pub fn render
  ( 
    &self, 
    gl : &GL, 
    textures : &
    Arc< 
      Mutex< 
        HashMap< 
          String, gl::web_sys::WebGlTexture
        > 
      > 
    >
  )
  {
    gl.use_program( Some( &self.material.program ) );

    // If material is present, then we loop over each texture this object uses
    // and bind them to a specific texture unit defined in material.rs
    if self.material.mtl.is_some()
    {
      let textures = textures.lock().unwrap();
      for ( i, t ) in self.material.texture_names.iter().enumerate()
      {
        if let Some( ref name ) = t
        {
          if let Some( texture ) = textures.get( name )
          {
            gl.active_texture( gl::TEXTURE0 + i as u32 );
            gl.bind_texture( gl::TEXTURE_2D, Some( &texture ) );
          }
        }
      }
    }

    gl.bind_vertex_array( Some( &self.vao ) );
    gl.draw_elements_with_i32( gl::TRIANGLES, self.indices_amount, gl::UNSIGNED_INT, 0 );
  }
}
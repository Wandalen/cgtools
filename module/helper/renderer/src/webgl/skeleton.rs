//!

mod private
{
  use minwebgl as gl;
  use gl::
  {
    GL,
    F32x4x4,
    web_sys::
    {
      js_sys::Float32Array,
      WebGlTexture,
      WebGlUniformLocation
    }
  };
  use crate::webgl::Node;
  use std::{ rc::Rc, cell::RefCell };
  use std::collections::HashMap;

  /// Joint matrices texture slot
  pub const JOINT_MATRICES_SLOT : u32 = 13;

  /// Loads data to data texture where every pixel
  /// is 4 float values. Used for packing matrices array
  fn load_texture_data_4f
  (
    gl : &GL,
    texture : &WebGlTexture,
    data : &[ f32 ],
    size : [ u32; 2 ],
  )
  {
    gl.active_texture( GL::TEXTURE0 );
    gl.bind_texture( GL::TEXTURE_2D, Some( texture ) );

    // Create a Float32Array from the Rust slice
    let js_data = Float32Array::from( data );

    let _ = gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_array_buffer_view
    (
      GL::TEXTURE_2D,
      0,
      GL::RGBA32F as i32,
      size[ 0 ] as i32,
      size[ 1 ] as i32,
      0,
      GL::RGBA,
      GL::FLOAT,
      Some( &js_data ),
    );

    gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_MIN_FILTER, GL::NEAREST as i32 );
    gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_MAG_FILTER, GL::NEAREST as i32 );

    gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_WRAP_S, GL::CLAMP_TO_EDGE as i32 );
    gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_WRAP_T, GL::CLAMP_TO_EDGE as i32 );
  }

  /// Binds a texture to a texture unit and uploads its location to a uniform.
  ///
  /// # Arguments
  ///
  /// * `gl` - The WebGL2 rendering context.
  /// * `texture` - The texture to bind.
  /// * `location` - The uniform location in the shader for the sampler.
  /// * `slot` - The texture unit to bind to ( e.g., `GL::TEXTURE0` ).
  fn upload_texture
  (
    gl : &GL,
    texture : &WebGlTexture,
    location : Option< WebGlUniformLocation >,
    slot : u32,
  )
  {
    gl.active_texture( gl::TEXTURE0 + slot );
    gl.bind_texture( GL::TEXTURE_2D, Some( &texture ) );
    // Tell the sampler uniform in the shader which texture unit to use ( 0 for GL_TEXTURE0, 1 for GL_TEXTURE1, etc. )
    gl.uniform1i( location.as_ref(), slot as i32 );
  }

  /// Set of virtual bones used to deform and control the
  /// movement of a 3D models. It's a fundamental concept
  /// in skeletal animation, the most common method for
  /// rigging and animating complex models.
  pub struct Skeleton
  {
    /// List of nodes name that is part of skeleton
    joints : Vec< Rc< RefCell< Node > > >,
    /// List of nodes correcting matrices used in nodes
    /// transform for playing skeletal animations
    inverse_bind_matrices :  Vec< F32x4x4 >,
    /// Joint matrices data texture
    texture : WebGlTexture
  }

  impl Skeleton
  {
    /// Create a new [`Skeleton`] instance
    ///
    /// # Arguments
    ///
    /// * `gl` - The WebGL2 rendering context.
    /// * `joints` - mapping names to inverse bind matrices of [`Skeleton`] nodes
    ///
    pub fn new
    (
      gl : &GL,
      joints : Vec< ( Rc< RefCell< Node > >, F32x4x4 ) >
    ) -> Option< Self >
    {
      let mut nodes = vec![];
      let mut inverse_bind_matrices = vec![];

      for ( node, matrix ) in joints
      {
        nodes.push( node );
        inverse_bind_matrices.push( matrix );
      }

      Some
      (
        Self
        {
          joints : nodes,
          inverse_bind_matrices : inverse_bind_matrices,
          texture : gl.create_texture()?,
        }
      )
    }

    /// Upload inverse bind matrices texture to current shader program
    pub fn upload
    (
      &self,
      gl : &GL,
      locations : &HashMap< String, Option< gl::WebGlUniformLocation > >
    )
    {
      let mut joint_matrices = vec![];

      for i in 0..self.inverse_bind_matrices.len()
      {
        let inverse_bind = self.inverse_bind_matrices[ i ];
        let node = &self.joints[ i ];
        let global_transform = node.borrow().get_world_matrix();
        joint_matrices.push( global_transform * inverse_bind );
      }

      let mut data = joint_matrices.iter()
      .map
      (
        | m | m.to_array().to_vec()
      )
      .flatten()
      .collect::< Vec< _ > >();

      let joint_matrices_loc = locations.get( "jointMatrices" ).unwrap();
      let texture_size_loc = locations.get( "jointMatricesSize" ).unwrap();

      let a = 4.0_f32.powf( ( data.len() as f32 ).sqrt().log( 4.0 ).ceil() ) as u32;
      let texture_size = [ a, a ];

      data.extend( vec![ 0.0; ( a * a * 4 ) as usize - data.len() ] );

      load_texture_data_4f( gl, &self.texture, data.as_slice(), texture_size );
      upload_texture( gl, &self.texture, joint_matrices_loc.clone(), JOINT_MATRICES_SLOT );
      gl::uniform::upload( &gl, texture_size_loc.clone(), texture_size.as_slice() ).unwrap();
    }
  }
}

crate::mod_interface!
{
  orphan use
  {
    Skeleton,
    JOINT_MATRICES_SLOT,
  };
}

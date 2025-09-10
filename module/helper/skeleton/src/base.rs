//!

mod private
{
  use minwebgl as gl;
  use gl::
  {
    GL,
    F32x4x4,
    WebGlBuffer
  };
  use std::collections::HashMap;

  /// Maximum joints count in skin that can support shader
  pub const MAX_JOINT_COUNT : i32 = 500;

  // /// Creates data texture where every pixel is 4 float values.
  // /// Used for packing uniform matrices array
  // fn create_texture_4f
  // (
  //   gl : &GL,
  //   data : &[ f32 ],
  //   size : [ u32; 2 ],
  // ) -> Option< WebGlTexture >
  // {
  //   // let Ok( _ ) = gl.get_extension( "OES_texture_float" )
  //   // else
  //   // {
  //   //   gl::error!( "skeleton crate: Failed to enable OES_texture_float extension" );
  //   //   return None;
  //   // };

  //   let texture = gl.create_texture();
  //   gl.active_texture( GL::TEXTURE0 );
  //   gl.bind_texture( GL::TEXTURE_2D, texture.as_ref() );

  //   // Create a Float32Array from the Rust slice
  //   let js_data = Float32Array::from( data );

  //   gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_array_buffer_view
  //   (
  //     GL::TEXTURE_2D,
  //     0,
  //     GL::RGBA32F as i32,
  //     size[ 0 ] as i32,
  //     size[ 1 ] as i32,
  //     0,
  //     GL::RGBA,
  //     GL::FLOAT,
  //     Some( &js_data ),
  //   )
  //   .ok()?;

  //   //gl.pixel_storei( GL::UNPACK_ALIGNMENT, 1 );

  //   gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_MIN_FILTER, GL::NEAREST as i32 );
  //   gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_MAG_FILTER, GL::NEAREST as i32 );

  //   gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_WRAP_S, GL::CLAMP_TO_EDGE as i32 );
  //   gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_WRAP_T, GL::CLAMP_TO_EDGE as i32 );

  //   texture
  // }

  // /// Binds a texture to a texture unit and uploads its location to a uniform.
  // ///
  // /// # Arguments
  // ///
  // /// * `gl` - The WebGL2 rendering context.
  // /// * `texture` - The texture to bind.
  // /// * `location` - The uniform location in the shader for the sampler.
  // /// * `slot` - The texture unit to bind to ( e.g., `GL::TEXTURE0` ).
  // fn upload_texture
  // (
  //   gl : &GL,
  //   texture : &WebGlTexture,
  //   location : Option< WebGlUniformLocation >,
  //   slot : u32,
  // )
  // {
  //   gl.active_texture( gl::TEXTURE0 + slot );
  //   gl.bind_texture( GL::TEXTURE_2D, Some( &texture ) );
  //   // Tell the sampler uniform in the shader which texture unit to use ( 0 for GL_TEXTURE0, 1 for GL_TEXTURE1, etc. )
  //   gl.uniform1i( location.as_ref(), slot as i32 );
  // }

  /// Set of virtual bones used to deform and control the
  /// movement of a 3D models. It's a fundamental concept
  /// in skeletal animation, the most common method for
  /// rigging and animating complex models.
  pub struct Skeleton
  {
    /// List of nodes name that is part of skeleton
    _joints : Vec< Box< str > >,
    /// List of nodes correcting matrices used in nodes
    /// transform for playing skeletal animations
    inverse_bind_matrices :  Vec< F32x4x4 >,
    // /// Size of [`Skeleton::texture`]
    // texture_size : [ u32; 2 ],
    // /// Inverse bind matrices data texture
    // texture : WebGlTexture
    /// InverseMatrices storage
    buffer : WebGlBuffer
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
      joints : HashMap< Box< str >, F32x4x4 >
    ) -> Option< Self >
    {
      let mut nodes = vec![];
      let mut inverse_bind_matrices = vec![];

      for ( name, matrix ) in joints
      {
        nodes.push( name );
        inverse_bind_matrices.push( matrix );
      }

      //minwebgl::info!( "Inverse bind matrices: {:#?}", inverse_bind_matrices );

      // let a = 4.0_f32.powf( ( data.len() as f32 ).sqrt().log( 4.0 ).ceil() ) as u32;
      // let texture_size = [ a, a ];
      // let texture = create_texture_4f( gl, data.as_slice(), texture_size ).unwrap();

      let joints_buffer = gl::buffer::create( &gl ).unwrap();

      Some
      (
        Self
        {
          _joints : nodes,
          inverse_bind_matrices : inverse_bind_matrices,
          buffer : joints_buffer
          // texture_size,
          // texture
        }
      )
    }

    /// Upload inverse bind matrices texture to current shader program
    pub fn upload
    (
      &self,
      gl : &GL
    )
    {
      // let inverse_matrices_loc = locations.get( "inverseMatrices" ).unwrap();
      // let texture_size_loc = locations.get( "inverseMatricesSize" ).unwrap();

      // upload_texture( gl, &self.texture, inverse_matrices_loc.clone(), slot );
      // gl::uniform::upload( &gl, texture_size_loc.clone(), self.texture_size.as_slice() ).unwrap();

      let data = self.inverse_bind_matrices.iter()
      .map
      (
        | m | m.to_array().to_vec()
      )
      .flatten()
      .collect::< Vec< _ > >();

      gl.bind_buffer_base( GL::UNIFORM_BUFFER, 0, Some( &self.buffer ) );
      gl.bind_buffer( GL::UNIFORM_BUFFER, Some( &self.buffer ) );
      gl.buffer_data_with_i32( GL::UNIFORM_BUFFER, MAX_JOINT_COUNT * 16 * 4, GL::STATIC_DRAW );
      gl::ubo::upload( &gl, &self.buffer, 0, &data[ .. ], GL::STATIC_DRAW );
    }
  }
}

crate::mod_interface!
{
  orphan use
  {
    Skeleton,
    MAX_JOINT_COUNT
  };
}

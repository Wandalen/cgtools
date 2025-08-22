//!

mod private
{
  use minwebgl as gl;
  use gl::
  { 
    web_sys::
    {
      WebGlProgram,
      js_sys::Float32Array
    },
    GL, 
    F32x4x4, 
    U32x2,
    WebGlUniformLocation
  };
  use std::{ cell::RefCell, collections::HashMap, rc::Rc };
  use animation::
  {
    interpolation::Transform,
    sequencer::Sequencer,
  };

  /// Creates data texture where every pixel is 4 float values. 
  /// Used for packing uniform matrices array
  fn create_texture_4f
  ( 
    gl : &GL,
    data : &[ f32 ], 
    size : U32x2
  ) -> Option< web_sys::WebGlTexture >
  {
    // let Ok( _ ) = gl.get_extension( "OES_texture_float" )
    // else
    // {
    //   gl::error!( "skeleton crate: Failed to enable OES_texture_float extension" );
    //   return None;
    // };

    let texture = gl.create_texture();
    gl.active_texture( GL::TEXTURE0 );
    gl.bind_texture( GL::TEXTURE_2D, texture.as_ref() );  

    // Create a Float32Array from the Rust slice
    let js_data = Float32Array::from( data );

    gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_array_buffer_view
    (
      GL::TEXTURE_2D,
      0,
      GL::RGBA32F as i32,
      size.x() as i32,
      size.y() as i32,
      0,
      GL::RGBA,
      GL::FLOAT,
      Some( &js_data ),
    )?;

    //gl.pixel_storei( GL::UNPACK_ALIGNMENT, 1 );

    gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_MIN_FILTER, GL::NEAREST as i32 );
    gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_MAG_FILTER, GL::NEAREST as i32 );

    gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_WRAP_S, GL::CLAMP_TO_EDGE as i32 );
    gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_WRAP_T, GL::CLAMP_TO_EDGE as i32 );

    texture
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
    gl.active_texture( slot ); 
    gl.bind_texture( GL::TEXTURE_2D, Some( &texture ) ); 
    // Tell the sampler uniform in the shader which texture unit to use ( 0 for GL_TEXTURE0, 1 for GL_TEXTURE1, etc. )
    gl.uniform1i( location.as_ref(), ( slot - GL::TEXTURE0 ) as i32 );
  }

  /// Set of virtual bones used to deform and control the 
  /// movement of a 3D models. It's a fundamental concept 
  /// in skeletal animation, the most common method for 
  /// rigging and animating complex models.
  pub struct Skeleton
  {
    /// List of nodes name that is part of skeleton
    joints : Vec< Box< str > >,
    /// List of nodes correcting matrices used in nodes 
    /// transform for playing skeletal animations 
    inverse_bind_matrices :  Vec< F32x4x4 >,
    // /// Current binded animation that controls transform 
    // /// of all joints in every time moment
    // animation : Option< Rc< RefCell< Sequencer > > >,
    /// Size of [`Skeleton::texture`]
    texture_size : U32x2,
    /// Inverse bind matrices data texture
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
    fn new
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

      let texture_size = U32x2::from_array( [ , ] );
      let data = ;

      let texture = create_texture_4f( gl, data, texture_size );

      Some
      ( 
        Self
        {
          joints : nodes,
          inverse_bind_matrices,
          // animation : None,
          texture_size,
          texture
        }
      )
    }

    /// Upload inverse bind matrices texture to current shader program
    fn upload
    ( 
      &self, 
      gl : &GL, 
      locations : HashMap< Box< str >, Option< WebGlUniformLocation > >,
      slot : u32
    ) 
    {
      let inverse_matrices_loc = locations.get( "inverseMatrices" ).unwrap();
      let texture_size_loc = locations.get( "inverseMatricesSize" ).unwrap();
      
      upload_texture( gl, self.texture, inverse_matrices_loc.clone(), slot );
      gl::uniform::upload( &gl, texture_size_loc.clone(), self.texture_size.as_slice() ).unwrap();
    }

    // /// Set current animation for this [`Skeleton`]
    // fn set_animation( &mut self, animation : Option< &Rc< RefCell< Sequencer > > > )
    // {
    //   self.animation = animation.cloned();
    // }

    // /// Return current animation of this [`Skeleton`]
    // fn get_animation( &self ) -> Option< Rc< RefCell< Sequencer > > >
    // {
    //   self.animation.clone()
    // }

    // /// Update current animation and get current nodes [`Transform`]'s
    // fn update( &self, time : f32 ) -> HashMap< Box< str >, Transform >
    // {
    //   let Some( animation ) = self.animation.as_ref()
    //   else
    //   {
    //     return HashMap::new();
    //   }

    //   animation.borrow_mut().
    // }
  }
}

crate::mod_interface!
{
  orphan use
  {
    Skeleton
  };
}
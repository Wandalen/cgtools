//!

mod private
{
  use bytemuck::cast_slice;
  use minwebgl as gl;

  use gl::{ GL, F32x4x4, U32x2 };
  use web_sys::
  {
    WebGlProgram,
    js_sys::Float32Array
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
    size : U32x2, 
    texture_id : u32 
  ) -> Option< web_sys::WebGlTexture >
  {
    // let Ok( _ ) = gl.get_extension( "OES_texture_float" )
    // else
    // {
    //   gl::error!( "skeleton crate: Failed to enable OES_texture_float extension" );
    //   return None;
    // };

    let texture = gl.create_texture();
    gl.active_texture( texture_id );
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

  /// 
  pub struct Skeleton
  {
    nodes : Vec< Box< str > >,
    inverse_bind_matrices :  Vec< F32x4x4 >,
    animation : Option< Rc< RefCell< Sequencer > > >,
    texture_size : U32x2,
    texture : WebGlTexture
  }

  impl Skeleton
  {
    /// 
    fn new
    ( 
      gl : &GL,
      joints : HashMap< Box< str >, F32x4x4 >,
      texture_id : u32
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

      let texture = create_texture_4f( gl, data, texture_size, texture_id );

      Some
      ( 
        Self
        {
          nodes,
          inverse_bind_matrices,
          animation : None,
          texture_size,
          texture
        }
      )
    }

    /// 
    fn set_animation( &mut self, animation : Option< &Rc< RefCell< Sequencer > > > )
    {
      self.animation = animation.cloned();
    }

    /// 
    fn get_animation( &self ) -> Option< Rc< RefCell< Sequencer > > >
    {
      self.animation.clone()
    }

    /// 
    fn update( &self, t : f32 ) -> HashMap< Box< str >, Transform >
    {
      
    }

    /// 
    fn upload( &self, gl : &GL, program : &WebGlProgram ) 
    {
      
    }
  }
}

crate::mod_interface!
{
  orphan use
  {
    Skeleton
  };
}
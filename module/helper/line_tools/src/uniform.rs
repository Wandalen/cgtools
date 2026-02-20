//! This file contains UniformStorage that helps you store and efficiently retrieve uniform data from your webgl program.

mod private
{
  use minwebgl::{self as gl, Vector };
  use gl::math::mat::{ DescriptorOrderColumnMajor, DescriptorOrderRowMajor, Mat2, Mat3, Mat4 };
  use rustc_hash::{ FxHashMap };

  macro_rules! impl_from_for_uniform 
  {
    ( $matrix_type:ident, $primitive_type:ty, $uniform_type:ident, true  ) => 
    {
      impl From< $matrix_type< $primitive_type, DescriptorOrderColumnMajor > > for Uniform
      {
        fn from( value: $matrix_type< $primitive_type, DescriptorOrderColumnMajor > ) -> Self 
        {
          Self::$uniform_type( value.to_array(), true )
        }
      }
    };

    ( $matrix_type:ident, $primitive_type:ty, $uniform_type:ident, false  ) => 
    {
      impl From< $matrix_type< $primitive_type, DescriptorOrderRowMajor > > for Uniform
      {
        fn from( value: $matrix_type< $primitive_type, DescriptorOrderRowMajor > ) -> Self 
        {
          Self::$uniform_type( value.to_array(), false )
        }
      }
    };

    ( $primitive_type:ty, $uniform_type:ident, $length:expr  ) => 
    {
      impl From< Vector< $primitive_type, $length > > for Uniform
      {
        fn from( value : Vector< $primitive_type, $length > ) -> Self 
        {
          Self::$uniform_type( value.0 )
        }
      }

      impl From< [ $primitive_type; $length ] > for Uniform
      {
        fn from( value : [ $primitive_type; $length ] ) -> Self 
        {
          Self::$uniform_type( value )
        }
      }
    };

    ( $primitive_type:ty, $uniform_type:ident  ) => 
    {
      impl From< $primitive_type > for Uniform
      {
        fn from( value: $primitive_type ) -> Self 
        {
          Self::$uniform_type( value )
        }
      }
    };
  }

  /// Enum to hold possible glsl uniform values
  #[ derive( Debug, Clone, Copy ) ]
  pub enum Uniform
  {
    /// float
    F32( f32 ),
    /// vec2
    F32x2( [ f32; 2 ] ),
    /// vec3
    F32x3( [ f32; 3 ] ),
    /// vec4
    F32x4( [ f32; 4 ] ),
    /// mat2
    F32x2x2( [ f32; 4 ], bool ),
    /// mat3
    F32x3x3( [ f32; 9 ], bool ),
    /// mat4
    F32x4x4( [ f32; 16 ], bool ),

    /// int
    I32( i32 ),
    /// ivec2
    I32x2( [ i32; 2 ] ),
    /// ivec3
    I32x3( [ i32; 3 ] ),
    /// ivec4
    I32x4( [ i32; 4 ] ),

    /// uint
    U32( u32 ),
    /// uvec2
    U32x2( [ u32; 2 ] ),
    /// uvec3
    U32x3( [ u32; 3 ] ),
    /// uvec4
    U32x4( [ u32; 4 ] ),
  }

  impl Uniform 
  {
    /// Upload the current value to the gl program at the specified location
    pub fn upload( &self, gl : &gl::WebGl2RenderingContext, location : Option< gl::WebGlUniformLocation > ) -> Result< (), gl::WebglError > 
    {
      match self  
      {
        Self::F32( v ) => gl::uniform::upload( gl, location, v )?,
        Self::F32x2( v ) => gl::uniform::upload( gl, location, v )?,
        Self::F32x3( v ) => gl::uniform::upload( gl, location, v )?,
        Self::F32x4( v ) => gl::uniform::upload( gl, location, v )?,
        Self::F32x2x2( v, c ) => gl::uniform::matrix_upload( gl, location, v, *c )?,
        Self::F32x3x3( v, c ) => gl::uniform::matrix_upload( gl, location, v, *c )?,
        Self::F32x4x4( v, c ) => gl::uniform::matrix_upload( gl, location, v, *c )?,

        Self::I32( v ) => gl::uniform::upload( gl, location, v )?,
        Self::I32x2( v ) => gl::uniform::upload( gl, location, v )?,
        Self::I32x3( v ) => gl::uniform::upload( gl, location, v )?,
        Self::I32x4( v ) => gl::uniform::upload( gl, location, v )?,

        Self::U32( v ) => gl::uniform::upload( gl, location, v )?,
        Self::U32x2( v ) => gl::uniform::upload( gl, location, v )?,
        Self::U32x3( v ) => gl::uniform::upload( gl, location, v )?,
        Self::U32x4( v ) => gl::uniform::upload( gl, location, v )?,
        //_ => {}
      }

      Ok( () )
    } 
  }

  /// Storage for uniforms and their locations in the gl program
  #[ derive( Debug, Default, Clone ) ]
  pub struct UniformStorage
  {
    /// Name - Value map of the uniforms
    uniforms : FxHashMap< Box< str >, Uniform >,
    /// Name - Location map of the uniforms
    locations : FxHashMap< Box< str >, gl::WebGlUniformLocation >
  }

  impl UniformStorage 
  {
    /// Add or update the value of a uniform with a specified name
    pub fn uniform_set< Name : Into< Box< str > > >( &mut self, name : Name, value : Uniform )
    {
      self.uniforms.insert( name.into(), value );
    }

    /// Upload the uniform with the specifed name to the gl program. If `use_locations` is true, looks for a saved uniform location of the specifed uniform in the map.
    /// If not found, asks for a location and adds it to the map for future use.
    pub fn upload
    ( 
      &mut self, 
      gl : &gl::WebGl2RenderingContext,
      program : &gl::WebGlProgram, 
      name : &str,
      use_locations : bool 
    ) -> Result< (), gl::WebglError >
    {
      if use_locations
      {
        upload_with_cache( gl, &self.uniforms, &mut self.locations, program, name )?;
      }
      else
      {
        upload_without_cache( gl, &self.uniforms, program, name )?
      }

      Ok( () )
    }

    /// Upload all uniforms to the program
    pub fn all_upload( &mut self, gl : &gl::WebGl2RenderingContext, program : &gl::WebGlProgram, use_locations : bool ) -> Result< (), gl::WebglError >
    {
      for name in self.uniforms.keys()
      {
        if use_locations
        {
          upload_with_cache( gl, &self.uniforms, &mut self.locations, program, name )?;
        }
        else
        {
          upload_without_cache( gl, &self.uniforms, program, name )?
        }
      }

      Ok( () )
    }

    /// Copy uniform to another UniformStorage
    pub fn copy_to( &self, other : &mut Self )
    {
      for ( name, value ) in self.uniforms.iter()
      {
        other.uniform_set( name.clone(), *value );
      }
    }

    /// Clear uniforms
    pub fn clear_uniforms( &mut self )
    {
      self.uniforms.clear();
    }

    /// Clear the locations cache
    pub fn clear_locations( &mut self )
    {
      self.locations.clear();
    }

  }

  fn upload_with_cache
  (
    gl : &gl::WebGl2RenderingContext,
    uniforms : &FxHashMap< Box< str >, Uniform >,
    cache : &mut FxHashMap< Box< str >, gl::WebGlUniformLocation >,
    program : &gl::WebGlProgram, 
    name : &str
  ) -> Result< (), gl::WebglError >
  {
    let uniform = uniforms.get( name ).ok_or( gl::WebglError::Other( "Uniform does not exist" ) )?;

    gl.use_program( Some( program ) );
    if let Some( location ) = cache.get( name )
    {
      uniform.upload( gl, Some( location.clone() ) )?;
    }
    else 
    {
      if let Some( location ) = gl.get_uniform_location( program, name )
      {
        cache.insert( name.into(), location.clone() );
        uniform.upload( gl, Some( location ) )?;
      }
    }

    Ok( () )
  }

  fn upload_without_cache
  (
    gl : &gl::WebGl2RenderingContext,
    uniforms : &FxHashMap< Box< str >, Uniform >,
    program : &gl::WebGlProgram, 
    name : &str
  ) -> Result< (), gl::WebglError >
  {
    let uniform = uniforms.get( name ).ok_or( gl::WebglError::Other( "Uniform does not exist" ) )?;

    gl.use_program( Some( program ) );
    if let Some( location ) = gl.get_uniform_location( program, name )
    {
      uniform.upload( gl, Some( location ) )?;
    }

    Ok( () )
  }

  impl_from_for_uniform!( f32, F32 );
  impl_from_for_uniform!( f32, F32x2, 2 );
  impl_from_for_uniform!( f32, F32x3, 3 );
  impl_from_for_uniform!( f32, F32x4, 4 );
  impl_from_for_uniform!( Mat2, f32, F32x2x2, true );
  impl_from_for_uniform!( Mat3, f32, F32x3x3, true );
  impl_from_for_uniform!( Mat4, f32, F32x4x4, true );
  impl_from_for_uniform!( Mat2, f32, F32x2x2, false );
  impl_from_for_uniform!( Mat3, f32, F32x3x3, false );
  impl_from_for_uniform!( Mat4, f32, F32x4x4, false );

  impl_from_for_uniform!( i32, I32 );
  impl_from_for_uniform!( i32, I32x2, 2 );
  impl_from_for_uniform!( i32, I32x3, 3 );
  impl_from_for_uniform!( i32, I32x4, 4 );

  impl_from_for_uniform!( u32, U32 );
  impl_from_for_uniform!( u32, U32x2, 2 );
  impl_from_for_uniform!( u32, U32x3, 3 );
  impl_from_for_uniform!( u32, U32x4, 4 );

  impl< T > From< &T > for Uniform
  where 
    T : Into< Uniform > + Copy
  {
    fn from( value: &T ) -> Self 
    {
      ( *value ).into()    
    }
  }
}

crate::mod_interface!
{
  orphan use
  {
    UniformStorage,
    Uniform
  };
}
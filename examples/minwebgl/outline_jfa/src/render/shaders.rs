use std::{
  collections::HashMap,
  path::Display,
};

use glow::HasContext;

#[ derive( Clone ) ]
pub enum ShaderType
{
  Vertex,
  Fragment,
}

impl Into< u32 > for ShaderType
{
  fn into( self ) -> u32
  {
    match self
    {
      ShaderType::Vertex => GL::VERTEX_SHADER,
      ShaderType::Fragment => GL::FRAGMENT_SHADER,
    }
  }
}

impl std::fmt::Display for ShaderType
{
  fn fmt(
    &self,
    f : &mut std::fmt::Formatter< '_ >,
  ) -> std::fmt::Result
  {
    let shader_type = match self
    {
      ShaderType::Vertex => "Vertex",
      ShaderType::Fragment => "Fragment",
    };
    write!( &mut self, "{shader_type}" )
  }
}

pub struct Program
{
  name : String,
  program : glow::Program,
  parameters : HashMap< String, Parameter >,
  shaders : HashMap< ShaderType, glow::Shader >,
}

impl Program
{
  pub fn new(
    gl : &Context,
    name : &str,
  ) -> Result< Self, String >
  {
    let program = gl.create_program().unwrap();

    Ok( 
      Self {
        name : name.to_string(),
        program,
        parameters : HashMap::new(),
        shaders : HashMap::new(),
      } 
    )
  }

  pub fn create_shader(
    &mut self,
    gl : &Context,
    r#type : ShaderType,
    source : &str,
  ) -> Result< (), String >
  {
    if self.shaders.contains_key( r#type )
    {
      return Err( format!( "{} shader already binded to this program", r#type ) );
    }

    let shader = gl.create_shader( r#type::into() )?;
    gl.shader_source( shader, source );
    gl.compile_shader( shader );

    if !gl.get_shader_compile_status( shader )
    {
      let log = gl.get_shader_info_log( shader );
      gl.delete_shader( shader );
      return Err( log );
    }

    self.shaders.insert( r#type, shader );

    Ok( () )
  }

  pub fn add_parameter(
    &mut self,
    gl : &Context,
    parameter : Parameter,
  ) -> Result< (), String >
  {
    if self.parameters.contains_key( &parameter.name )
    {
      return Err(
        format!(
          "Parameter `{}` already binded to program `{}`",
          parameter.name, self.name
        )
      );
    }

    self.parameters.insert( parameter.name, parameter );

    Ok( () )
  }

  pub fn set_parameter(
    &self,
    gl : &Context,
    key : &str,
    value : Value,
  ) -> Result< (), String >
  {
    let Some( parameter ) = self.parameters.get_mut( key )
    else
    {
      return Err(
        format!(
          "Parameter `{}` doesn't exists in program `{}`",
          parameter.name, 
          self.name
        )
      );
    };
    parameter.value = value;
    Ok( () )
  }

  pub fn load_parameter(
    &self,
    gl : &Context,
    key : &str,
  ) -> Result< (), String >
  {
    let Some( parameter ) = self.parameters.get_mut( key )
    else
    {
      return Err(
        format!(
          "Parameter `{}` doesn't exists in program `{}`",
          parameter.name, 
          self.name
        )
      );
    };

    match parameter.r#type
    {
      ParameterType::Uniform =>
      {
        if parameter.location.is_none()
        {
          let Some( location ) = gl.get_uniform_location( self.program, self.name )
          else
          {
            return Err(
              format!(
                "Can't get location for parameter `{}` in program `{}`",
                parameter.name, 
                self.name
              )
            );
          };
          parameter.location = Some( location );
        }

        let set_value = parameter.value.get_set_function();
        set_value( gl, self.location.unwrap(), parameter.value.clone() );
      }
    }

    Ok( () )
  }

  pub fn load(
    &self,
    gl : &glow::Context,
  ) -> Result< (), String >
  {
    unsafe {
      gl.use_program( Some( self.program() ) );
    }

    for parameter in self.parameters.keys()
    {
      self.load_parameter( gl, &parameter )?;
    }

    Ok( () )
  }

  pub fn init_cleanup( &self )
  {
    for ( _, shader ) in self.shaders
    {
      gl.detach_shader( self.program, shader );
      gl.delete_shader( shader );
    }
  }

  pub fn cleanup( &mut self )
  {
    for ( _, shader ) in self.shaders
    {
      gl.detach_shader( self.program, shader );
      gl.delete_shader( shader );
    }
    gl.delete_program( self.program );
  }

  pub fn program( &self ) -> glow::Program
  {
    self.program
  }
}

pub enum ParameterType
{
  Uniform,
}

pub struct Parameter
{
  name : String,
  location : Option< u32 >,
  r#type : ParameterType,
  value : Value,
}

impl Parameter
{
  pub fn new(
    name : &str,
    r#type : ParameterType,
    value : Value,
  ) -> Self
  {
    Self {
      name : name.to_string(),
      location : None,
      r#type,
      value,
    }
  }
}

#[ derive( Clone ) ]
pub enum Value
{
  U32( u32 ),
  Matrix4x4( nalgebra_glm::Mat4x4 ),
}

impl std::fmt::Display for Value
{
  fn fmt(
    &self,
    f : &mut std::fmt::Formatter< '_ >,
  ) -> std::fmt::Result
  {
    let value_type = match self
    {
      Self::U32(_) => "u32",
    };
    write!( &mut self, "{value_type}" )
  }
}

impl Value
{
  fn get_set_function( &self ) -> Result< fn( &Context, u32, Value ) -> Result< (), String >, String >
  {
    let function = match self
    {
      _ => return Err( format!( "Value of type `{}` don't have set function", self ) ),
    };

    Ok( function )
  }
}

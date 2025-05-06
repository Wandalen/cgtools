use std::{
  collections::HashMap,
  mem::discriminant,
  path::Display,
  cell::RefCell,
  rc::Rc
};

use minwebgl::{
  web_sys::{
    WebGlFramebuffer,
    WebGlProgram,
    WebGlShader,
    WebGlTexture,
    WebGlUniformLocation
  },
  GL,
};
use ndarray_cg::mat::DescriptorOrderColumnMajor;
use web_sys::{WebGlBuffer, WebGlVertexArrayObject};

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
  id : WebGlProgram,
  parameters : HashMap< String, Parameter >,
  shaders : HashMap< ShaderType, WebGlShader >,
  linked : bool,
  vbo : WebGlBuffer,
  vao : WebGlVertexArrayObject
}

impl Program
{
  pub fn new( 
    gl : &GL,
    name : &str,
  ) -> Result< Self, String >
  {
    let Some( id ) = gl.create_program()
    else
    {
      return Err( format!( "Can't create program `{}`", name ) );
    };

    let Some( vbo ) = gl.create_buffer()
    else
    {
      return Err( format!( "Can't create vertex buffer for program `{}`", name ) );
    };

    let Some( vao ) = gl.create_vertex_array()
    else
    {
      return Err( format!( "Can't create vertex array buffer for program `{}`", name ) );
    };

    Ok( 
      Self {
        name : name.to_string(),
        id,
        parameters : HashMap::new(),
        shaders : HashMap::new(),
        linked : false,
        vbo,
        vao
      } 
    )
  }

  pub fn create_shader( 
    &mut self,
    gl : &GL,
    r#type : ShaderType,
    source : &str,
  ) -> Result< (), String >
  {
    if self.shaders.contains_key( r#type )
    {
      return Err( 
        format!( 
          "{} shader already binded to program `{}`",
          r#type, 
          self.name
        ) 
      );
    }

    let Some( shader ) = gl.create_shader( r#type::into() )
    else
    {
      return Err( 
        format!( 
          "Can't create {} shader for program `{}`",
          r#type, 
          self.name
        ) 
      );
    };
    gl.shader_source( &shader, source );
    gl.compile_shader( &shader );

    if let Some( log ) = gl.get_shader_info_log( &shader )
    {
      gl.delete_shader( shader );
      return Err( log );
    }

    self.shaders.insert( r#type, shader );

    Ok( () )
  }

  pub fn add_parameter( 
    &mut self,
    gl : &GL,
    parameter : Parameter,
  ) -> Result< (), String >
  {
    if self.parameters.contains_key( &parameter.name )
    {
      return Err( 
        format!( 
          "Parameter `{}` already binded to program `{}`",
          parameter.name, 
          self.name
        ) 
      );
    }

    if let ParameterType::Input = parameter.r#type
    {
      if self.linked
      {
        return Err( 
          format!( 
            "Can't add parameter `{}`, because program `{}` is linked",
            parameter.name, 
            self.name
          ) 
        );
      }

      let Some( location ) = parameter.location
      else
      {
        return Err( 
          format!( 
            "Parameter `{}` doesn't have location in program `{}`",
            parameter.name, 
            self.name
          ) 
        );
      };

      let Value::AttribArray( ads, _ ) = parameter.value
      else
      {
        return Err( 
          format!( 
            "Parameter `{}` isn`t attribute array, because it has invalid type (`{}`) in program `{}`",
            parameter.name,
            parameter.r#type,
            self.name
          ) 
        );
      };

      for ( location, ad ) in ads.into_iter().enumerate()
      {
        gl.bind_attrib_location( &self.id, location, ad.name.as_str() );
      }
    }

    self.parameters.insert( parameter.name, parameter );

    Ok( () )
  }

  pub fn set_parameter( 
    &self,
    gl : &GL,
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

    if discriminant( &parameter.value ) != discriminant( &value )
    {
      return Err( 
        format!( 
          "Parameter `{}` has value type `{}`, not `{}`",
          parameter.name, 
          parameter.value, 
          value
        ) 
      );
    }
    parameter.value = value;
    Ok( () )
  }

  pub fn load_parameter( 
    &self,
    gl : &GL,
    key : &str,
  ) -> Result< (), String >
  {
    let err = Err( 
      format!( 
        "Parameter `{}` with type `{}` has wrong value type ( `{}` ) in program `{}`",
        parameter.name, 
        parameter.name, 
        parameter.value, 
        self.name
      ) 
    );

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
        self.set_location( gl, parameter.name )?;
        let set_value = parameter.get_set_function();
        set_value( gl, self.location.unwrap(), parameter.value.clone() );
      }
      ParameterType::Texture =>
      {
        self.set_location( gl, parameter.name )?;
        let Value::Texture( texture ) = parameter.value
        else
        {
          return err;
        };
        &texture.borrow().load( gl, parameter.location )?;
      }
      ParameterType::Framebuffer =>
      {
        let Value::Framebuffer( framebuffer ) = &parameter.value
        else
        {
          return err;
        };

        gl.bind_framebuffer( GL::FRAMEBUFFER, Some( &framebuffer.id() ) );
        let ( width, height ) = framebuffer.color.size;
        gl.viewport( 0, 0, width, height );
      }
      ParameterType::Input => self.load_input( gl, parameter.name.as_str() )?,
      _ =>
      {
        return Err( 
          format!( 
            "Can't load parameter `{}` with type `{}`",
            parameter.name, 
            parameter.r#type
          ) 
        )
      }
    }

    Ok( () )
  }

  fn load_input( &self, gl : &GL, name : &str ) -> Result< (), String >
  {
    let err = Err( 
      format!( 
        "Can't load attrib array `{}` in program `{}`",
        name,
        self.name 
      ) 
    );
    
    let Some( parameter ) = self.parameters.get( name )
    else
    {
      return err;
    };

    let Value::AttribArray( ads, data ) = parameter.value
    else
    {
      return err;
    };

    let mut stride = 0;
    for ad in ads{
      stride += ad.size;
    }
 
    gl.bind_vertex_array( Some( &self.vao ) );
    gl.bind_buffer( GL::ARRAY_BUFFER, Some( &self.vbo ) );
    gl.buffer_data_with_u8_array( GL::ARRAY_BUFFER, &data, GL::STATIC_DRAW );
    let mut offset = 0;
    for ( location, ad ) in ads.into_iter().enumerate(){
      gl.enable_vertex_attrib_array( location );
      gl.vertex_attrib_pointer_with_i32( 
        location,
        ad.size,
        ad.r#type,
        false,
        stride,
        offset
      );
      offset += ad.size;
    }

    Ok( () )
  }

  pub fn load( 
    &self,
    gl : &GL,
  ) -> Result< (), String >
  {
    self.unload( gl );

    gl.use_program( Some( &self.id() ) );

    for parameter in self.parameters.keys()
    {
      self.load_parameter( gl, &parameter )?;
    }

    Ok( () )
  }

  pub fn unload( 
    &self,
    gl : &GL,
  )
  {
    gl.bind_vertex_array( None );
    gl.bind_buffer(GL::ARRAY_BUFFER, None );
    gl.use_program( None );
  }

  pub fn link( 
    &mut self,
    gl : &GL,
  ) -> Result< (), String >
  {
    gl.link_program( &program.id );

    if let Some( log ) = gl.get_program_info_log( &self.id )
    {
      self.cleanup( gl );
      return Err( log );
    }

    for ( _, shader ) in self.shaders
    {
      gl.detach_shader( &self.id, &shader );
      gl.delete_shader( Some( &shader ) );
    }

    self.linked = true;

    Ok( () )
  }

  pub fn cleanup( 
    &mut self,
    gl : &GL,
  )
  {
    let err = Err( 
      format!( 
        "Parameter `{}` with type `{}` has wrong value type (  `{}`  ) in program `{}`",
        parameter.name, 
        parameter.name, 
        parameter.value, 
        self.name
      ) 
    );

    for ( _, shader ) in self.shaders
    {
      gl.detach_shader( &self.id, &shader );
      gl.delete_shader( Some( &shader ) );
    }
    gl.delete_program( Some( &self.id ) );

    for ( _, parameter ) in self.parameters
    {
      match parameter.r#type
      {
        ParameterType::Input =>
        {}
        ParameterType::Texture =>
        {
          let Value::Texture( texture ) = parameter.value
          else
          {
            return err;
          };
          gl.delete_texture( &texture.borrow().id() );
        }
        ParameterType::Framebuffer =>
        {
          let Value::Framebuffer( framebuffer ) = &parameter.value
          else
          {
            return err;
          };
          gl.delete_framebuffer( Some( &framebuffer.id() ) );
        }
        _ => (),
      }
    }
  }

  pub fn id( &self ) -> WebGlProgram
  {
    self.id.clone()
  }

  fn set_location( 
    &mut self,
    gl : &GL,
    parameter_name : String,
  ) -> Result< (), String >
  {
    if !self.linked
    {
      return Err( 
        format!( 
          "Can't set location for parameter `{}`, because program `{}` isn't yet linked",
          parameter.name, 
          self.name
        ) 
      );
    }

    let Some( parameter ) = self.parameters.get_mut( parameter_name.as_str() )
    else
    {
      return Err( 
        format!( 
          "Parameter `{}` already binded to program `{}`",
          parameter_name, self.name
        ) 
      );
    };

    if parameter.location.is_none()
    {
      let location = gl.get_uniform_location( &self.id, parameter_name.as_str() );
      let Some( location ) = location
      else
      {
        return Err( 
          format!( 
            "Can't get location for parameter `{}` in program `{}`",
            parameter_name, 
            self.name
          ) 
        );
      };
      parameter.set_location( gl, location )?;
    }
    Ok( () )
  }

  fn add_parameters(
    &mut self, 
    gl : &GL,
    r#type : ParameterType, 
    parameters : HashMap< String, Value >,
  ) -> Result<(), String>
  {
    for ( name, value ) in parameters
    {
      let parameter = Parameter::new( &name, r#type, value );
      self.add_parameter( gl, parameter )?;
    }

    Ok( () )
  }

  fn set_parameters(
    &mut self, 
    gl : &GL, 
    parameters : HashMap< String, Value >,
  ) -> Result<(), String>
  {
    for ( name, value ) in parameters
    {
      self.set_parameter( gl, &name, value )?;
    }

    Ok( () )
  }
}

impl Program
{
  pub fn add_uniform( &mut self, gl : &GL, name : &self, value : Value ) -> Result<(), String>
  {
    self.add_parameter( gl, Parameter::new( name, ParameterType::Uniform, value ) )
  }

  pub fn add_input( &mut self, gl : &GL, name : &self, value : Value ) -> Result<(), String>
  {
    self.add_parameter( gl, Parameter::new( name, ParameterType::Input, value ) )
  }

  pub fn add_texture( &mut self, gl : &GL, name : &self, texture : Rc< RefCell< Texture > > ) -> Result<(), String>
  {
    self.add_parameter( gl, Parameter::new( name, ParameterType::Texture, Value::Texture( texture ) ) )
  }

  pub fn add_framebuffer( &mut self, gl : &GL, name : &self, fb : Framebuffer ) -> Result<(), String>
  {
    self.add_parameter( gl, Parameter::new( name, ParameterType::Framebuffer, Value::Framebuffer( fb ) ) )
  }
}

pub struct Framebuffer
{
  id : WebGlFramebuffer,
  color : WebGlTexture,
  depth : Option< WebGlTexture >,
}

impl Framebuffer
{
  pub fn new( 
    gl : &GL,
    color : WebGlTexture,
    depth : Option< WebGlTexture >,
  ) -> Result< (), String >
  {
    let Some( id ) = gl.create_framebuffer()
    else
    {
      return Err( "Can't create framebuffer".to_string() );
    };
    gl.bind_framebuffer( GL::FRAMEBUFFER, Some( &id ) );

    gl.framebuffer_texture_2d( 
      GL::FRAMEBUFFER,
      GL::COLOR_ATTACHMENT0,
      GL::TEXTURE_2D,
      Some( &color ),
      0,
    );

    if let Some( depth ) = depth
    {
      gl.framebuffer_texture_2d( 
        GL::FRAMEBUFFER,
        GL::DEPTH_ATTACHMENT,
        GL::TEXTURE_2D,
        Some( &depth ),
        0,
      );
    }

    let status = gl.check_framebuffer_status( GL::FRAMEBUFFER );
    if status != GL::FRAMEBUFFER_COMPLETE
    {
      let status_str = match status
      {
        GL::FRAMEBUFFER_UNSUPPORTED => "UNSUPPORTED",
        GL::FRAMEBUFFER_INCOMPLETE_ATTACHMENT => "INCOMPLETE_ATTACHMENT",
        GL::FRAMEBUFFER_INCOMPLETE_MISSING_ATTACHMENT => "INCOMPLETE_MISSING_ATTACHMENT",
        GL::FRAMEBUFFER_INCOMPLETE_MULTISAMPLE => "INCOMPLETE_MULTISAMPLE",
        GL::FRAMEBUFFER_INCOMPLETE_DIMENSIONS => "INCOMPLETE_DIMENSIONS",
        _ => "Unknown",
      };
      gl.bind_framebuffer( GL::FRAMEBUFFER, None );
      gl.delete_framebuffer( Some( &id ) );
      Err( format!( "Status: {} (`{}`)", status_str, status ) )
    }
    else
    {
      gl.bind_framebuffer( GL::FRAMEBUFFER, None );
      Ok( 
        Self 
        {
          id,
          color,
          depth,
        } 
      )
    }
  }
}

enum TextureType
{
  Texture2D,
}

pub struct Texture
{
  r#type : TextureType,
  id : WebGlTexture,
  slot : u32,
  size : ( usize, usize ),
  internal_format : u32,
  format : u32,
  pixel_type : u32,
  filter : u32,
  data : Option< Vec< u8 > >,
}

impl Texture
{
  pub fn new( 
    gl : &GL,
    slot : u32,
    size : ( usize, usize ),
    internal_format : u32,
    format : u32,
    pixel_type : u32,
  ) -> Result< Self, String >
  {
    if slot  > 31
    {
      return Err( "Slot can't be bigger than 31".to_string() );
    }

    let Some( id ) = id
    else
    {
      return Err( "Can't create texture".to_string() );
    };

    let mut texture = Self {
      r#type : TextureType::Texture2D,
      id,
      slot,
      size,
      internal_format,
      format,
      pixel_type,
      filter,
      data : None,
    };

    Ok( texture )
  }

  fn set_data( 
    &mut self,
    data : Option< Vec< u8 > >,
  ) -> Result< (), String >
  {
    if todo!()
    {
      return Err( todo!() );
    }

    self.data = data;

    Ok( () )
  }

  fn load( 
    &self,
    gl : &GL,
    location : &WebGlUniformLocation,
  ) -> Result< (), String >
  {
    gl.active_texture( self.slot_code() );
    gl.bind_texture( self.r#type, Some( &self.id ) );

    match self.r#type
    {
      TextureType::Texture2D =>
      {
        gl.bind_texture( GL::TEXTURE_2D, Some( &self.id ) );
        gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array( 
          GL::TEXTURE_2D,
          0,
          self.internal_format as i32,
          self.size.0,
          self.size.1,
          0,
          self.format,
          self.pixel_type,
          self.data.as_ref(),
        );
      }
    }

    gl.uniform1i( Some( location ), self.slot );

    Ok( () )
  }

  fn r#type( &self ) -> TextureType
  {
    self.r#type
  }

  pub fn id( &self ) -> u32
  {
    self.id
  }

  fn slot( &self ) -> u32
  {
    self.slot
  }

  fn slot_code( &self ) -> u32
  {
    if self.slot <  32
    {
      33_984u32 + self.slot
    }
    else
    {
      unreachable!()
    }
  }
}

#[ derive( Copy, Clone ) ]
pub enum ParameterType
{
  Input,
  Uniform,
  Texture,
  Framebuffer,
}

impl std::fmt::Display for ParameterType
{
  fn fmt( 
    &self,
    f : &mut std::fmt::Formatter< '_ >,
  ) -> std::fmt::Result
  {
    let parameter_type = match self
    {
      Self::Input => "Input",
      Self::Uniform => "Uniform",
      Self::Texture => "Texture",
      Self::Framebuffer => "Framebuffer",
    };
    write!( &mut self, "{value_type}" )
  }
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

  fn get_set_function( &self ) -> Result< fn( &GL, u32, Value ) -> Result< (), String >, String >
  {
    let err = Err( 
      format!( 
        "Parameter `{}` with type `{}` doesn't have set function for value of type `{}`",
        self.name, 
        self.r#type, 
        self.value
      ) 
    );
    let function = match self.r#type
    {
      ParameterType::Input =>
      {
        match self.value
        {
          _ => return err,
        }
      }
      ParameterType::Uniform =>
      {
        match self.value
        {
          Value::U32( _ ) => set_functions::set_uniform_scalar,
          Value::Matrix4x4( _ ) => set_functions::set_uniform_matrix,
          _ => return err,
        }
      }
      ParameterType::Texture =>
      {
        match self.value
        {
          _ => return err,
        }
      }
      ParameterType::Framebuffer => todo!(),
    };

    Ok( function )
  }

  fn set_location( 
    &mut self,
    gl : &GL,
    location : u32,
  ) -> Result< (), String >
  {
    if parameter.location.is_some()
    {
      return Err( format!( "Parameter `{}` has already location", self.name ) );
    }

    self.location = Some( location );
    Ok( () )
  }
}

#[ derive( Clone ) ]
pub struct AttribData
{
  name : String,
  size : usize,
  r#type : u32,
}

impl AttribData 
{
  pub fn new(   
    name : &str,
    size : usize,
    r#type : u32, 
  ) -> Self 
  {
    Self
    {
      name : name.to_string(),
      size,
      r#type,
    }
  }
}

#[ derive( Clone ) ]
pub enum Value
{
  U32( u32 ),
  Matrix4x4( ndarray_cg::Mat4< f32, DescriptorOrderRowMajor > ),
  Texture( Rc< RefCell< Texture > > ),
  Framebuffer( Framebuffer ),
  AttribArray( Vec< AttribData >, Vec< u8 > )
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
        Self::U32( _ ) => "u32",
        Self::Matrix4x4( matrix ) => todo!(),
        Self::Texture( texture ) => todo!(),
        Self::Framebuffer( framebuffer ) => todo!(),
        Self::AttribArray( attrib_datas, items ) => todo!(),
    };
    write!( &mut self, "{value_type}" )
  }
}

mod set_functions
{
  pub fn set_uniform_scalar( 
    gl : &GL,
    location : u32,
    value : Value,
  ) -> Result< (), String >
  {
    match value
    {
      Value::U32( v ) => gl.uniform1u( location, v ),
      _ =>
      {
        Err( 
          format!( 
            "set_uniform_scalar doesn't support value of type `{}`",
            v
          ) 
        )
      }
    };

    Ok( () )
  }

  pub fn set_uniform_vec( 
    gl : &GL,
    location : u32,
    value : Value,
  ) -> Result< (), String >
  {
    match value
    {
      Value::U32( v ) => gl.uniform1u( location, v ),
      _ =>
      {
        Err( 
          format!( 
            "set_uniform_vec doesn't support value of type `{}`",
            v
          ) 
        )
      }
    };

    Ok( () )
  }

  pub fn set_uniform_matrix( 
    gl : &GL,
    location : u32,
    value : Value,
  ) -> Result< (), String >
  {
    match value
    {
      Value::U32( v ) => gl.uniform1u( location, v ),
      _ =>
      {
        Err( 
          format!( 
            "set_uniform_matrix doesn't support value of type `{}`",
            v
          ) 
        )
      }
    };

    Ok( () )
  }
}

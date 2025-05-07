mod camera;
mod shaders;

use crate::input::InputState;

use minwebgl::{
  texture, web_sys::{
    HtmlCanvasElement,
    WebGlUniformLocation,
  }, Mat4, COLOR_ATTACHMENT0, GL
};
use ndarray_cg::mat::DescriptorOrderRowMajor;
use shaders::*;
use web_sys::WebGlTexture;
use std::{
  collections::HashMap,
  sync::LazyLock,
  cell::RefCell,
  rc::Rc
};

// Pass 1: 3D Object Rendering
const OBJECT_VS : &str = include_str!( "./shaders/object.vert" );

const OBJECT_FS : &str = include_str!( "./shaders/object.frag" );

// Pass 2 & 3 & 4: Fullscreen Quad Vertex Shader
const FULLSCREEN_VS : &str = include_str!( "./shaders/fullscreen.vert" );

// Pass 2: JFA Initialization Fragment Shader
const JFA_INIT_FS : &str = include_str!( "./shaders/jfa_init.frag" );

// Pass 3: JFA Step Fragment Shader
const JFA_STEP_FS : &str = include_str!( "./shaders/jfa_step.frag" );

// Pass 4: Final Outline Composite Fragment Shader
const OUTLINE_FS : &str = include_str!( "./shaders/outline.frag" );

static PROGRAM_INIT_FNS : LazyLock< Vec< fn( &mut Renderer ) -> Result< (), String > > > =
  LazyLock::new( || vec![ init_textures, object, jfa_init, jfa_step, outline ] );

pub struct Renderer
{
  viewport : Viewport,
  camera : Camera,
  context : Rc< RefCell< GL > >,
  programs : HashMap< String, Program >,
  textures : HashMap< String, Rc< RefCell< Texture > > >
}

impl Renderer
{
  pub fn new( 
    context : GL,
    viewport : Viewport,
  ) -> Result< Self, String >
  {
    let mut renderer = Self {
      viewport,
      camera : todo!(),
      context: Rc::new( RefCell::new( context ) ),
      programs : HashMap::new(),
      textures : HashMap::new()
    };
    
    for program_init_fn in &*PROGRAM_INIT_FNS
    {
      program_init_fn( &mut renderer )?;
    }

    Ok( renderer )
  }

  pub fn update( 
    &mut self,
    input_state : InputState,
   ) -> Result< (), String >
  {
    if let Some( timestamp ) = input_state.timestamp
    {
      let program_name = "object";
      let Some( object ) = self.programs.get_mut( program_name )
      else
      {
        return Err( format!( "Can't find program `{}`", program_name ) );
      };
      object.set_parameter( "time", Value::U32( timestamp ) )?;
    }

    Ok( () )
  }

  pub fn render( &mut self )
  {
    let object = self.programs.get( "object" ).unwrap();

    object.load();

    let jfa_init = self.programs.get( "jfa_init" ).unwrap();

    jfa_init.load();

    let jfa_step = self.programs.get( "jfa_step" ).unwrap();

    jfa_step.load();

    let outline = self.programs.get( "outline" ).unwrap();

    outline.load();
  }

  pub fn cleanup( &mut self )
  {
    for ( _, program ) in self.programs.iter_mut()
    {
      program.cleanup();
    }
  }

  fn get_mut_program( &mut self, program_name : &str ) -> Option< &mut Program >
  {
    self.programs.get_mut( program_name )
  }

  fn add_program( &mut self, program: Program ) -> Result< (), String >
  {
    if self.programs.contains_key( program.name().as_str() )
    {
      return Err( format!( "Program with name `{}` already exists", program.name() ) );
    };
    self.programs.insert( program_name.to_string(), program );
    
    Ok( () )
  }

  fn add_texture( &mut self, texture : Texture ) -> Result< (), String > 
  {
    let name = texture.name();
    if self.textures.contains_key( name.as_str() )
    {
      return Err( format!( "Texture with name `{}` already exists", name ) );
    };
    let texture = Rc::new( RefCell::new( texture ) );
    self.textures.insert( name, texture );

    Ok( () )
  }

  fn get_texture( &mut self, texture_name : &str ) -> Some( Rc< RefCell< Texture > > )
  {
    self.textures.get( texture_name )
  }

  fn add_texture_to_program( &mut self, program_name : &str, texture_name : &str ) -> Result< (), String > 
  {
    let Some( program ) = self.programs.get_mut( program_name )
    else
    {
      return Err( format!( "Can't find program `{}`", program_name ) );
    };

    let Some( texture ) = self.textures.get( texture_name )
    else
    {
      return Err( format!( "Can't find program `{}`", texture_name ) );
    };

    program.add_texture( texture )
  } 

  fn create_framebuffer( &self, name : &str, color_attachment : u32, color : &str, depth : Option< &str > ) -> Result< Framebuffer, String > 
  {
    let Some( color ) = self.get_texture( color ) 
    else
    {
      return Err( format!( "Can't find texture `{}`", color ) );
    };
    let color_id = color.borrow().id();

    let mut depth_id = None;

    if let Some( depth ) = depth
    {
      let Some( d ) = self.get_texture( depth ) 
      else
      {
        return Err( format!( "Can't find texture `{}`", depth ) );
      };
      depth_id = Some( d.borrow().id() );
    }

    let gl = renderer.context.borrow();
    Framebuffer::new( &gl, name, color_attachment, color_id, depth_id )
  }
}

pub struct Viewport
{
  width : u32,
  height : u32,
}

impl Viewport
{
  pub fn new( 
    width : u32,
    height : u32,
   ) -> Self
  {
    Self {
      width,
      height,
    }
  }
}

fn init_textures( renderer : &mut Renderer ) -> Result< (), String >
{
  let gl = renderer.context.borrow();
  let v = renderer.viewport;

  let textures = 
  [
    Texture::new( &gl, "object_fb_color", 0, ( v.width, v.height ), GL::RGBA8, GL::RGBA, GL::UNSIGNED_BYTE )?,
    Texture::new( &gl, "object_fb_depth", 1, ( v.width, v.height ), GL::DEPTH_COMPONENT24, GL::DEPTH_COMPONENT, GL::UNSIGNED_INT )?,
    Texture::new( &gl, "jfa_init_fb_color", 2, ( v.width, v.height ), GL::RGBA8, GL::RGBA, GL::UNSIGNED_BYTE )?,
    Texture::new( &gl, "jfa_init_fb_depth", 3, ( v.width, v.height ), GL::DEPTH_COMPONENT24, GL::DEPTH_COMPONENT, GL::UNSIGNED_INT )?,
    Texture::new( &gl, "jfa_step_fb_color", 4, ( v.width, v.height ), GL::RGBA8, GL::RGBA, GL::UNSIGNED_BYTE )?,
    Texture::new( &gl, "jfa_step_fb_depth", 5, ( v.width, v.height ), GL::DEPTH_COMPONENT24, GL::DEPTH_COMPONENT, GL::UNSIGNED_INT )?,
  ];

  for texture in textures 
  {
    renderer.add_texture( texture )?;
  }

  Ok( () )
}

fn object( renderer : &mut Renderer ) -> Result< (), String >
{
  let program_name = "object";

  let mut program = Program::new( &renderer.context, program_name )?;

  program.create_shader( ShaderType::Vertex, OBJECT_VS )?;
  program.create_shader( ShaderType::Fragment, OBJECT_FS )?;

  let quad_vertices: [f32; 12] = [ -1.0, -1.0, 1.0, -1.0, -1.0, 1.0, -1.0, 1.0, 1.0, -1.0, 1.0, 1.0 ];
  let data = unsafe{
    core::slice::from_raw_parts( quad_vertices.as_ptr() as *const u8, quad_vertices.len() * core::mem::size_of::<f32>(), );
  };

  program.add_input( 
    "0",  
    vec![
      AttribData::new( "a_pos", 3, GL::FLOAT )          
    ], 
    data 
  )?;

  program.link()?;
  renderer.add_program( program )?;

  renderer.add_texture_to_program( program_name, "object_fb_color" )?;
  renderer.add_texture_to_program( program_name, "object_fb_depth" )?;

  let program = renderer.get_mut_program( program_name ).unwrap();

  let mvp = Mat4::< f32, DescriptorOrderRowMajor >::default();
  let object_fb = renderer.create_framebuffer( "object_fb", 0, "object_fb_color", Some( "object_fb_depth" ) )?;
  
  program.add_uniform( "mvp", Value::Matrix4x4( mvp ) )?;
  program.add_framebuffer( object_fb )?;

  Ok( () )
}

fn jfa_init( renderer : &mut Renderer ) -> Result< (), String >
{
  let program_name = "jfa_init";

  let mut program = Program::new( &renderer.context, program_name )?;
  let gl = renderer.context.borrow();

  program.create_shader( ShaderType::Vertex, FULLSCREEN_VS )?;
  program.create_shader( ShaderType::Fragment, JFA_INIT_FS )?;

  let quad_vertices: [f32; 12] = [ -1.0, -1.0, 1.0, -1.0, -1.0, 1.0, -1.0, 1.0, 1.0, -1.0, 1.0, 1.0 ];
  let data = unsafe{
    core::slice::from_raw_parts( quad_vertices.as_ptr() as *const u8, quad_vertices.len() * core::mem::size_of::<f32>(), );
  };

  program.add_input( 
    "0",  
    vec![
      AttribData::new( "a_pos", 3, GL::FLOAT )          
    ], 
    data 
  )?;

  program.link()?;
  renderer.add_program( program )?;

  for texture in [ "object_fb_color", "jfa_init_fb_color", "jfa_init_fb_depth" ]
  {
    renderer.add_texture_to_program( program_name, texture )?;
  } 

  let program = renderer.get_mut_program( program_name ).unwrap();

  let jfa_init_fb = renderer.create_framebuffer( "jfa_init_fb", 0, "jfa_init_fb_color", Some( "jfa_init_fb_depth" ) )?;
  
  program.add_framebuffer( jfa_init_fb )?;
  program.add_uniform( 
    "u_resolution",
    Value::Vec2( ndarray_cg::U32x2::from_slice( &[ v.width, v.height ] ) )
  );

  Ok( () )
}

fn jfa_step( renderer : &mut Renderer ) -> Result< (), String >
{
  let program_name = "jfa_step";

  let mut program = Program::new( &renderer.context, program_name )?;

  program.create_shader( ShaderType::Vertex, FULLSCREEN_VS )?;
  program.create_shader( ShaderType::Fragment, JFA_STEP_FS )?;

  let quad_vertices: [f32; 12] = [ -1.0, -1.0, 1.0, -1.0, -1.0, 1.0, -1.0, 1.0, 1.0, -1.0, 1.0, 1.0 ];
  let data = unsafe{
    core::slice::from_raw_parts( quad_vertices.as_ptr() as *const u8, quad_vertices.len() * core::mem::size_of::<f32>(), );
  };

  program.add_input( 
    "0",  
    vec![
      AttribData::new( "a_pos", 3, GL::FLOAT )          
    ], 
    data 
  )?;

  program.link()?;
  renderer.add_program( program )?;

  for texture in [ "jfa_init_fb_color", "jfa_step_fb_color", "jfa_step_fb_depth" ]
  {
    renderer.add_texture_to_program( program_name, texture )?;
  } 

  let program = renderer.get_mut_program( program_name ).unwrap();

  let jfa_step_fb = renderer.create_framebuffer( "jfa_step_fb", 0, "jfa_step_fb_color", Some( "jfa_step_fb_depth" ) )?;
  
  let v = renderer.viewport;
  program.add_framebuffer( jfa_step_fb )?;
  program.add_uniform( 
    "u_resolution",
    Value::Vec2( ndarray_cg::U32x2::from_slice( &[ v.width, v.height ] ) )
  );
  program.add_uniform( 
    "u_step_size",
    Value::F32( todo!() )
  );

  Ok( () )
}

fn outline( renderer : &mut Renderer ) -> Result< (), String >
{
  let program_name = "outline";

  let mut program = Program::new( &renderer.context, program_name )?;

  program.create_shader( ShaderType::Vertex, FULLSCREEN_VS )?;
  program.create_shader( ShaderType::Fragment, OUTLINE_FS )?;

  let quad_vertices: [f32; 12] = [ -1.0, -1.0, 1.0, -1.0, -1.0, 1.0, -1.0, 1.0, 1.0, -1.0, 1.0, 1.0 ];
  let data = unsafe{
    core::slice::from_raw_parts( quad_vertices.as_ptr() as *const u8, quad_vertices.len() * core::mem::size_of::<f32>(), );
  };

  program.add_input( 
    "0",  
    vec![
      AttribData::new( "a_pos", 3, GL::FLOAT )          
    ], 
    data 
  )?;

  program.link()?;
  renderer.add_program( program )?;

  for texture in [ "jfa_init_fb_color", "jfa_step_fb_color", "jfa_step_fb_depth" ]
  {
    renderer.add_texture_to_program( program_name, texture )?;
  }

  let program = renderer.get_mut_program( program_name ).unwrap();

  program.add_uniform( 
    "u_resolution",
    Value::Vec2( ndarray_cg::U32x2::from_slice( &[ v.width, v.height ] ) )
  );

  program.add_uniform( 
    "u_outline_thickness",
    Value::F32( todo!() )
  );

  program.add_uniform( 
    "u_outline_color",
    Value::Vec4( todo!() )
  );

  program.add_uniform( 
    "u_object_color",
    Value::Vec4( todo!() )
  );

  program.add_uniform( 
    "u_background_color",
    Value::Vec4( todo!() )
  );

  Ok( () )
}

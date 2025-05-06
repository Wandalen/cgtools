mod camera;
mod shaders;

use crate::input::InputState;

use minwebgl::{
  web_sys::{
    HtmlCanvasElement,
    WebGlUniformLocation,
  }, 
  Mat4, 
  GL
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
  LazyLock::new( || vec![ object, jfa_init, jfa_step, outline ] );

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
      object.set_parameter( &self.context, "time", Value::U32( timestamp ) )?;
    }

    Ok( () )
  }

  pub fn render( &mut self )
  {
    let object = self.programs.get( "object" ).unwrap();

    object.load( &self.context );

    let jfa_init = self.programs.get( "jfa_init" ).unwrap();

    jfa_init.load( &self.context );

    let jfa_step = self.programs.get( "jfa_step" ).unwrap();

    jfa_step.load( &self.context );

    let outline = self.programs.get( "outline" ).unwrap();

    outline.load( &self.context );
  }

  pub fn cleanup( &mut self )
  {
    for ( _, program ) in self.programs.iter_mut()
    {
      program.cleanup( &self.context );
    }
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

fn object( renderer : &mut Renderer ) -> Result< (), String >
{
  let mut program = Program::new( gl, "object" )?;

  program.create_shader( gl, ShaderType::Vertex, OBJECT_VS )?;
  program.create_shader( gl, ShaderType::Fragment, OBJECT_FS )?;

  let quad_vertices: [f32; 12] = [ -1.0, -1.0, 1.0, -1.0, -1.0, 1.0, -1.0, 1.0, 1.0, -1.0, 1.0, 1.0 ];
  let data = unsafe{
    core::slice::from_raw_parts( quad_vertices.as_ptr() as *const u8, quad_vertices.len() * core::mem::size_of::<f32>(), );
  };

  let v = renderer.viewport;
  let object_fb_color = Texture::new( gl, 0, ( v.width, v.height ), GL::RGBA8, GL::RGBA, GL::UNSIGNED_BYTE )?;
  let object_fb_depth = Texture::new( gl, 1, ( v.width, v.height ), GL::DEPTH_COMPONENT24, GL::DEPTH_COMPONENT, GL::UNSIGNED_INT )?;
  let attrib_datas = vec![
    AttribData::new( "a_pos", 3, GL::FLOAT )          
  ];
  let mvp = Mat4::< f32, DescriptorOrderRowMajor >::default();
  let object_fb = Framebuffer::new( gl, object_fb_color.id(), Some( object_fb_depth.id() ) );

  program.add_input( gl, "0", Value::AttribArray( attrib_datas, data ) );

  program.link( gl )?;

  program.add_uniform( gl, "mvp", Value::Matrix4x4( mvp ) );
  program.add_framebuffer( gl, "object_fb", object_fb );

  let object_fb_color = Rc::new( RefCell::new( object_fb_color ) );
  program.add_texture( gl, "object_fb_color", object_fb_color );
  renderer.textures.insert( "object_fb_color".to_string(), object_fb_color );

  let object_fb_depth = Rc::new( RefCell::new( object_fb_depth ) );
  program.add_texture( gl, "object_fb_depth", object_fb_depth );
  renderer.textures.insert( "object_fb_depth".to_string(), object_fb_depth );

  renderer.programs.insert( program_name.to_string(), program );

  Ok( () )
}

fn jfa_init( renderer : &mut Renderer ) -> Result< (), String >
{
  let program_name = "jfa_init" ;

  let mut program = Program::new( gl, program_name )?;

  program.create_shader( gl, ShaderType::Vertex, FULLSCREEN_VS )?;
  program.create_shader( gl, ShaderType::Fragment, JFA_INIT_FS )?;

  let quad_vertices: [f32; 12] = [ -1.0, -1.0, 1.0, -1.0, -1.0, 1.0, -1.0, 1.0, 1.0, -1.0, 1.0, 1.0 ];
  let data = unsafe{
    core::slice::from_raw_parts( quad_vertices.as_ptr() as *const u8, quad_vertices.len() * core::mem::size_of::<f32>(), );
  };

  program.add_input( 
    gl,
    "0",
    Value::AttribArray( 
      vec![
        AttribData::new( "a_pos", 3, GL::FLOAT )          
      ], 
      data 
    ),
  );

  program.link( gl )?;

  program.add_texture( 
    gl,
    "u_object_texture",
    Texture::new(

    )
  );

  program.add_uniform( 
    gl,
    "u_resolution",
    Value::Vec2( ndarray_cg::U32x2::from_slice( &[ 1920, 1080 ] ) )
  );

  renderer.programs.insert( program_name.to_string(), program );

  Ok( () )
}

fn jfa_step( renderer : &mut Renderer ) -> Result< (), String >
{
  let mut program = Program::new( gl, "jfa_step" )?;

  program.create_shader( gl, ShaderType::Vertex, FULLSCREEN_VS )?;
  program.create_shader( gl, ShaderType::Fragment, JFA_STEP_FS )?;

  let quad_vertices: [f32; 12] = [ -1.0, -1.0, 1.0, -1.0, -1.0, 1.0, -1.0, 1.0, 1.0, -1.0, 1.0, 1.0 ];
  let data = unsafe{
    core::slice::from_raw_parts( quad_vertices.as_ptr() as *const u8, quad_vertices.len() * core::mem::size_of::<f32>(), );
  };

  program.add_input( 
    gl,
    "0",
    Value::AttribArray( 
      vec![
        AttribData::new( "a_pos", 3, GL::FLOAT )          
      ], 
      data 
    ),
  );

  program.link( gl )?;

  renderer.programs.insert( program_name.to_string(), program );

  Ok( () )
}

fn outline( renderer : &mut Renderer ) -> Result< (), String >
{
  let mut program = Program::new( gl, "outline" )?;

  program.create_shader( gl, ShaderType::Vertex, FULLSCREEN_VS )?;
  program.create_shader( gl, ShaderType::Fragment, OUTLINE_FS )?;

  let quad_vertices: [f32; 12] = [ -1.0, -1.0, 1.0, -1.0, -1.0, 1.0, -1.0, 1.0, 1.0, -1.0, 1.0, 1.0 ];
  let data = unsafe{
    core::slice::from_raw_parts( quad_vertices.as_ptr() as *const u8, quad_vertices.len() * core::mem::size_of::<f32>(), );
  };

  program.add_input( 
    gl,
    "0",
    Value::AttribArray( 
      vec![
        AttribData::new( "a_pos", 3, GL::FLOAT )          
      ], 
      data 
    ),
  );

  program.link( gl )?;

  renderer.programs.insert( program_name.to_string(), program );

  Ok( () )
}

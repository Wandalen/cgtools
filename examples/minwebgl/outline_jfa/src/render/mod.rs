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
use std::{
  collections::HashMap,
  sync::LazyLock,
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

static PROGRAM_INIT_FNS : LazyLock< Vec< fn( &GL ) -> Result< Program, String > > > =
  LazyLock::new( || vec![ object, jfa_init, jfa_step, outline ] );

pub struct Renderer
{
  viewport : Viewport,
  camera : Camera,
  context : GL,
  programs : HashMap< String, Program >,
}

impl Renderer
{
  pub fn new( 
    context : GL,
    viewport : Viewport,
  ) -> Result< Self, String >
  {
    let mut programs = HashMap::new();
    for ( name, program_init_fn ) in &*PROGRAM_INIT_FNS
    {
      programs.insert( name.clone(), program_init_fn( &context )? );
    }

    let mut renderer = Self {
      viewport,
      camera : todo!(),
      context,
      programs,
    };

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

fn object( gl : &GL ) -> Result< Program, String >
{
  let mut program = Program::new( gl, "object" )?;

  program.create_shader( gl, ShaderType::Vertex, OBJECT_VS )?;
  program.create_shader( gl, ShaderType::Fragment, OBJECT_FS )?;

  program.add_parameter( 
    gl,
    Parameter::new( 
      "a_pos",
      ParameterType::Input,
      Value::Matrix4x4( Mat4::<f32, DescriptorOrderRowMajor>::default() ),
     ),
  );

  program.link( gl )?;

  program.add_parameter( 
    gl,
    Parameter::new( 
      "mvp",
      ParameterType::Uniform,
      Value::Matrix4x4( Mat4::<f32, DescriptorOrderRowMajor>::default() ),
     ),
  );

  Ok( program )
}

fn jfa_init( gl : &GL ) -> Result< Program, String >
{
  let mut program = Program::new( gl, "jfa_init" )?;

  program.create_shader( gl, ShaderType::Vertex, FULLSCREEN_VS )?;
  program.create_shader( gl, ShaderType::Fragment, JFA_INIT_FS )?;

  program.link( gl )?;

  Ok( program )
}

fn jfa_step( gl : &GL ) -> Result< Program, String >
{
  let mut program = Program::new( gl, "jfa_step" )?;

  program.create_shader( gl, ShaderType::Vertex, FULLSCREEN_VS )?;
  program.create_shader( gl, ShaderType::Fragment, JFA_STEP_FS )?;

  program.link( gl )?;

  Ok( program )
}

fn outline( gl : &GL ) -> Result< Program, String >
{
  let mut program = Program::new( gl, "outline" )?;

  program.create_shader( gl, ShaderType::Vertex, FULLSCREEN_VS )?;
  program.create_shader( gl, ShaderType::Fragment, OUTLINE_FS )?;

  program.link( gl )?;

  Ok( program )
}

use minwebgpu::{self as gl, web_sys, WebGPUError};
use rand::Rng;

pub const NUM_LIGHTS : usize = 150;

#[ repr( C ) ]
#[derive( Default, Clone, Copy, gl::mem::Pod, gl::mem::Zeroable ) ]
pub struct LightRaw
{
  color : [ f32; 3 ],
  power : f32,
  position : [ f32; 3 ],
  direction : f32
}

pub struct Light
{
  pub color : gl::F32x3,
  pub power : f32,
  pub position : gl::F32x3,
  pub direction : f32
}

impl Light
{
  pub fn as_raw( &self ) -> LightRaw
  {
    LightRaw
    {
      power : self.power,
      color : self.color.to_array(),
      position : self.position.to_array(),
      direction : self.direction,
      ..Default::default()
    }
  }
}

pub struct LightState
{
  pub buffer : gl::web_sys::GpuBuffer
}

impl LightState
{
  pub fn new( device : &web_sys::GpuDevice ) -> Result< Self, WebGPUError >
  {
    let lights = generate_lights();
    let lights_raw = lights.iter().map( | l | l.as_raw() ).collect::< Vec< LightRaw> >();

    let buffer = gl::BufferInitDescriptor::new
    (
       &lights_raw,
       gl::BufferUsage::STORAGE | gl::BufferUsage::VERTEX
    ).create( device )?;

    Ok
    (
      LightState
      {
        buffer
      }
    )
  }
}

pub struct LightVisualizationState
{
  pub render_pipeline : web_sys::GpuRenderPipeline
}

impl LightVisualizationState
{
  pub fn vertex_layout() -> web_sys::GpuVertexBufferLayout
  {
    gl::layout::VertexBufferLayout::new()
    .instance()
    .stride::< LightRaw >()
    .attribute
    (
      gl::layout::VertexAttribute::new()
      .location( 0 )
      .offset::< [ f32; 4 ] >()
      .format( gl::GpuVertexFormat::Float32x3 )
    )
    .attribute
    (
      gl::layout::VertexAttribute::new()
      .location( 1 )
      .format( gl::GpuVertexFormat::Float32x3 )
    )
    .into()
  }

  pub fn new( device : &web_sys::GpuDevice, format : web_sys::GpuTextureFormat ) -> Result< Self, WebGPUError >
  {
    let shader_module = gl::shader::create( device, include_str!( "../shaders/light.wgsl" ) );

    let render_pipeline = gl::render_pipeline::desc
    (
      gl::VertexState::new( &shader_module )
      .buffer( &Self::vertex_layout() )
    )
    .fragment
    (
      gl::FragmentState::new( &shader_module )
      .target( gl::ColorTargetState::new().format( format ) )
    )
    .primitive( gl::PrimitiveState::new().triangle_strip() )
    .depth_stencil( gl::DepthStencilState::new() )
    .create( device )?;

    Ok
    (
      LightVisualizationState
      {
        render_pipeline
      }
    )
  }
}

fn generate_lights() -> Vec< Light >
{
  let mut rng = rand::thread_rng();

  let mut lights = Vec::new();
  for _i in 0..NUM_LIGHTS
  {
    let power = rng.gen::< f32 >() * 2.0 + 1.0;
    let color = gl::F32x3::from( [ rng.gen(), rng.gen(), rng.gen() ] );
    let direction = if rng.gen::< f32 >() < 0.5 { -1.0 } else { 1.0 };

    let mut position = gl::F32x3::from
    ([
      rng.gen::< f32 >() * 2.0,
      rng.gen::< f32 >(),
      rng.gen::< f32 >() * 2.0
    ]) - gl::F32x3::from( [ 1.0, 0.0, 1.0 ] ) ;

    position = position * gl::F32x3::from( [ 40.0, 5.0, 40.0 ] );

    let light = Light
    {
      power,
      color,
      position,
      direction
    };

    lights.push( light );
  }

  lights
}

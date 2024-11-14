use minwebgpu::{self as gl, web_sys, WebGPUError};
use rand::Rng;

#[ repr( C ) ]
#[derive( Default, Clone, Copy, gl::mem::Pod, gl::mem::Zeroable ) ]
pub struct LightRaw
{
  color : [ f32; 3 ],
  phase : f32,
  velocity : [ f32; 3 ],
  power : f32
}

pub struct Light
{
  pub color : gl::F32x3,
  pub phase : f32,
  pub velocity : gl::F32x3,
  pub power : f32
}

impl Light 
{
  pub fn as_raw( &self ) -> LightRaw
  {
    LightRaw
    {
      power : self.power,
      color : self.color.to_array(),
      velocity : self.velocity.to_array(),
      phase : self.phase,
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
    let lights = gen();
    let buffer = gl::BufferInitDescriptor::new( &lights, gl::BufferUsage::STORAGE ).create( device )?;
    Ok
    ( 
      LightState
      {
        buffer
      }
    )
  }  
}

fn gen() -> Vec< LightRaw >
{
  let mut rng = rand::thread_rng();
  
  let mut lights = Vec::new();
  for _i in 0..300
  {
    let power = rng.gen::< f32 >() * 2.0;
    let color = gl::F32x3::from( [ rng.gen(), rng.gen(), rng.gen() ] );
    let velocity = ( gl::F32x3::from( [ rng.gen(), rng.gen(), rng.gen() ] ) * 2.0 - 1.0 ) * 100.0;
    let phase = rng.gen::< f32 > () * 2.0 * 3.1415926;

    let light = Light
    {
      power,
      color,
      velocity,
      phase
    };

    lights.push( light );
  }

  let lights = lights.iter().map( | l | l.as_raw() ).collect();
  lights
}
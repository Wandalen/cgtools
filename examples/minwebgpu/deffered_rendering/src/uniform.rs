use minwebgpu::{self as gl, WebGPUError};
use gl::web_sys;
#[ repr( C ) ]
#[ derive( Clone, Copy, Default, gl::mem::Pod, gl::mem::Zeroable ) ]
pub struct UniformRaw
{
  pub view_matrix : [ f32; 16 ],
  pub projection_matrix : [ f32; 16 ],
  pub camera_pos : [ f32; 3 ],
  pub time : f32,
  pub elapsed_time : f32,
  _padding : [ f32; 3 ]
}

#[ derive( Default ) ]
pub struct Uniform
{
  pub view_matrix : gl::math::F32x4x4,
  pub projection_matrix : gl::math::F32x4x4,
  pub camera_pos : gl::math::F32x3,
  pub time : f32,
  pub elapsed_time : f32
}

impl Uniform
{
  pub fn as_raw( &self ) -> UniformRaw
  {
    UniformRaw
    {
      view_matrix : self.view_matrix.to_array(),
      projection_matrix : self.projection_matrix.to_array(),
      camera_pos : self.camera_pos.to_array(),
      time : self.time,
      elapsed_time : self.elapsed_time,
      ..Default::default()
    }
  }
}

pub struct UniformState
{
  pub uniform  : Uniform,
  pub buffer : web_sys::GpuBuffer
}

impl UniformState
{
  pub fn new( device: &web_sys::GpuDevice ) -> Result< Self, WebGPUError >
  {
    let uniform = Uniform::default();
    let buffer = gl::BufferDescriptor::new( gl::BufferUsage::UNIFORM | gl::BufferUsage::COPY_DST )
    .size::< UniformRaw >()
    .create( device )?;

    Ok
    (
      UniformState
      {
        uniform,
        buffer
      }
    )
  }

  pub fn update( &self, queue : &web_sys::GpuQueue ) -> Result< (), WebGPUError >
  {
    gl::queue::write_buffer( queue, &self.buffer, &[ self.uniform.as_raw() ] )?;
    Ok( () )
  }
}
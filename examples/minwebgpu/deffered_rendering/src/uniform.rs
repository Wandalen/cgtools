use minwebgpu::{self as gl, WebGPUError};
use gl::web_sys;

#[ repr( C ) ]
#[ derive( Clone, Copy, bytemuck::NoUninit, Default ) ]
pub struct UniformRaw
{
  pub view_matrix : [ f32; 16 ],
  pub projection_matrix : [ f32; 16 ],
  pub camera_pos : [ f32; 3 ],
  pub _padding : [ f32; 1 ]
}


pub struct Uniform
{
  view_matrix : gl::math::Mat4< f32 >,
  projection_matrix : gl::math::Mat4< f32 >,
  camera_pos : gl::math::F32x3
}

pub struct UniformState
{
  pub buffer : web_sys::GpuBuffer,
  pub bind_group : web_sys::GpuBindGroup,
  pub bind_group_layout : web_sys::GpuBindGroupLayout
}

impl UniformState 
{
  pub fn new( device: &web_sys::GpuDevice ) -> Result< Self, WebGPUError >
  {
    let buffer = gl::BufferDescriptor::new( gl::BufferUsage::UNIFORM | gl::BufferUsage::COPY_DST )
    .size::< UniformRaw >()
    .create( device )?;

    let bind_group_layout = gl::BindGroupLayoutDescriptor::new()
    .all()
    .auto_bindings()
    .entry_from_ty( gl::binding_type::buffer() )
    .create( device )?;

    let bind_group = gl::BindGroupDescriptor::new( &bind_group_layout )
    .auto_bindings()
    .entry_from_resource( &gl::BufferBinding::new( &buffer ) )
    .create( device );

    Ok
    (
      UniformState
      {
        buffer,
        bind_group,
        bind_group_layout
      }
    )
  } 

  pub fn update( &self, queue : &web_sys::GpuQueue,  raw : UniformRaw ) -> Result< (), WebGPUError >
  {
    gl::queue::write_buffer( queue, &self.buffer, raw )?;
    Ok( () )
  }   
}
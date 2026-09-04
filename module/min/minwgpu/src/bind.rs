//! This module provides helpers for creating common bind group arrangements,
//! reducing the boilerplate of writing layout and bind group descriptors by hand.

use mingl::mod_interface;

mod private
{
  /// Creates a bind group layout and bind group exposing `buffer` as the single
  /// uniform at `binding = 0`, visible to the given shader stages.
  ///
  /// This covers the common one-uniform-struct setup : the returned layout plugs into a
  /// pipeline layout, the returned bind group into a render pass.
  #[ must_use ]
  pub fn single_uniform
  (
    device : &wgpu::Device,
    buffer : &wgpu::Buffer,
    visibility : wgpu::ShaderStages
  )
  -> ( wgpu::BindGroupLayout, wgpu::BindGroup )
  {
    let layout = device.create_bind_group_layout
    (
      &wgpu::BindGroupLayoutDescriptor
      {
        label : Some( "single_uniform_bind_group_layout" ),
        entries :
        &[
          wgpu::BindGroupLayoutEntry
          {
            binding : 0,
            visibility,
            ty : wgpu::BindingType::Buffer
            {
              ty : wgpu::BufferBindingType::Uniform,
              has_dynamic_offset : false,
              min_binding_size : None,
            },
            count : None,
          },
        ],
      }
    );

    let bind_group = device.create_bind_group
    (
      &wgpu::BindGroupDescriptor
      {
        label : Some( "single_uniform_bind_group" ),
        layout : &layout,
        entries :
        &[
          wgpu::BindGroupEntry
          {
            binding : 0,
            resource : buffer.as_entire_binding(),
          },
        ],
      }
    );

    ( layout, bind_group )
  }
}

mod_interface!
{
  own use single_uniform;
}

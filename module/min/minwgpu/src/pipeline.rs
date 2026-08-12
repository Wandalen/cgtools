//! This module provides helpers for creating common render pipeline shapes,
//! starting with the bufferless-triangle pipeline used by shader-driven
//! offscreen examples.

use mingl::mod_interface;

mod private
{
  /// Creates a render pipeline for the bufferless-triangle pattern : `vs_main` derives
  /// positions from the vertex index ( no vertex buffers ), `fs_main` shades into a
  /// single color target of the given `format` with [`wgpu::BlendState::REPLACE`].
  ///
  /// The WGSL source must define `vs_main` and `fs_main` entry points. Bind group
  /// layouts in `layouts` become the pipeline layout; pass an empty slice when the
  /// shader binds nothing. Primitive, multisample and depth state are the `wgpu`
  /// defaults ( triangle list, no multisampling, no depth ).
  #[ must_use ]
  pub fn fullscreen
  (
    device : &wgpu::Device,
    wgsl : &str,
    format : wgpu::TextureFormat,
    layouts : &[ &wgpu::BindGroupLayout ]
  )
  -> wgpu::RenderPipeline
  {
    let shader = device.create_shader_module
    (
      wgpu::ShaderModuleDescriptor
      {
        label : Some( "fullscreen_pipeline_shader" ),
        source : wgpu::ShaderSource::Wgsl( wgsl.into() ),
      }
    );

    // wgpu 30 : layout entries are `Option`al and push constants became `immediate_size`.
    let layouts : Vec< Option< &wgpu::BindGroupLayout > > = layouts.iter().copied().map( Some ).collect();
    let layout = device.create_pipeline_layout
    (
      &wgpu::PipelineLayoutDescriptor
      {
        label : Some( "fullscreen_pipeline_layout" ),
        bind_group_layouts : &layouts,
        immediate_size : 0
      }
    );

    device.create_render_pipeline
    (
      &wgpu::RenderPipelineDescriptor
      {
        label : Some( "fullscreen_pipeline" ),
        layout : Some( &layout ),
        vertex : wgpu::VertexState
        {
          module : &shader,
          entry_point : Some( "vs_main" ),
          compilation_options : wgpu::PipelineCompilationOptions::default(),
          buffers : &[]
        },
        primitive : wgpu::PrimitiveState::default(),
        depth_stencil : None,
        multisample : wgpu::MultisampleState::default(),
        fragment : Some
        (
          wgpu::FragmentState
          {
            module : &shader,
            entry_point : Some( "fs_main" ),
            compilation_options : wgpu::PipelineCompilationOptions::default(),
            targets :
            &[
              Some
              (
                wgpu::ColorTargetState
                {
                  format,
                  blend : Some( wgpu::BlendState::REPLACE ),
                  write_mask : wgpu::ColorWrites::ALL
                }
              )
            ]
          }
        ),
        multiview_mask : None,
        cache : None
      }
    )
  }
}

mod_interface!
{
  own use fullscreen;
}

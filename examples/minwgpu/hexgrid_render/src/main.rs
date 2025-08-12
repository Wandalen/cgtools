//! Example of rendering a hexagonal grid with wgpu

fn main()
{
  pollster::block_on( async { run().await } );
}

async fn run()
{
  let instance = wgpu::Instance::new
  (
    &wgpu::InstanceDescriptor
    {
      backends: wgpu::Backends::PRIMARY,
      ..Default::default()
    }
  );

  let adapter = instance.request_adapter
  (
    &wgpu::RequestAdapterOptionsBase
    {
      power_preference : wgpu::PowerPreference::HighPerformance,
      ..Default::default()
    }
  ).await.expect( "Failed to retrieve an adapter" );

  let ( device, queue ) = adapter.request_device
  (
    &wgpu::wgt::DeviceDescriptor::default()
  ).await.expect( "Failed to retrieve a device" );

  let shader = device.create_shader_module( wgpu::include_wgsl!( "../shaders/shader.wgsl" ) );

  let width = 512;
  let height = 512;
  let texture_extent = wgpu::Extent3d
  {
    width,
    height,
    depth_or_array_layers : 1,
  };
  let texture = device.create_texture
  (
    &wgpu::TextureDescriptor
    {
      label : Some( "Render Texture" ),
      size : texture_extent,
      mip_level_count : 1,
      sample_count : 1,
      dimension : wgpu::TextureDimension::D2,
      format : wgpu::TextureFormat::Rgba8UnormSrgb,
      usage : wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
      view_formats : &[],
    }
  );
  let texture_view = texture.create_view( &wgpu::TextureViewDescriptor::default() );

  let render_pipeline_layout = device.create_pipeline_layout
  (
    &wgpu::PipelineLayoutDescriptor
    {
      label : Some( "hexagonal_pipeline_layout" ),
      bind_group_layouts : &[],
      push_constant_ranges : &[]
    }
  );

  let render_pipeline = device.create_render_pipeline
  (
    &wgpu::RenderPipelineDescriptor
    {
      label : Some( "hexagonal_pipeline" ),
      layout : Some( &render_pipeline_layout ),
      vertex: wgpu::VertexState
      {
        module : &shader,
        entry_point : Some( "vs_main" ),
        compilation_options : wgpu::PipelineCompilationOptions::default(),
        buffers : &[]
      },
      primitive : wgpu::PrimitiveState
      {
        topology : wgpu::PrimitiveTopology::TriangleList,
        strip_index_format : None,
        front_face : wgpu::FrontFace::Ccw,
        cull_mode : None,
        unclipped_depth : false,
        polygon_mode : wgpu::PolygonMode::Fill,
        conservative : false
      },
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
                format : wgpu::TextureFormat::Rgba8UnormSrgb,
                blend : Some( wgpu::BlendState::REPLACE ),
                write_mask : wgpu::ColorWrites::ALL
              }
            )
          ]
        }
      ),
      multiview : None,
      cache : None
    }
  );

}

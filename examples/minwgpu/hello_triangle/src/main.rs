#![ doc = "../readme.md" ]

fn main()
{
  run();
}

fn run()
{
  let ( device, queue ) = create_device_and_queue();
  let render_pipeline = create_render_pipeline( &device );

  let width = 512;
  let height = 512;
  let ( texture, texture_view, output_buffer, texture_extent ) = create_render_target( &device, width, height );

  let mut encoder = device.create_command_encoder( &wgpu::CommandEncoderDescriptor { label : Some( "encoder" ) } );
  render_triangle( &mut encoder, &render_pipeline, &texture_view );

  let bytes_per_pixel = 4;
  encoder.copy_texture_to_buffer
  (
    texture.as_image_copy(),
    wgpu::TexelCopyBufferInfo
    {
      buffer : &output_buffer,
      layout : wgpu::TexelCopyBufferLayout
      {
        offset : 0,
        bytes_per_row : Some( width * bytes_per_pixel ),
        rows_per_image : None
      }
    },
    texture_extent
  );

  queue.submit( Some( encoder.finish() ) );

  let buffer_slice = output_buffer.slice( .. );
  buffer_slice.map_async( wgpu::MapMode::Read, | _ | {} );

  device.poll( wgpu::PollType::Wait { submission_index : None, timeout : None } ).expect( "Failed to render an image" );

  let data = buffer_slice.get_mapped_range();
  image::save_buffer( "-triangle.png", &data, width, height, image::ColorType::Rgba8 )
  .expect( "Failed to save image" );
}

fn create_device_and_queue() -> ( wgpu::Device, wgpu::Queue )
{
  let instance = wgpu::Instance::new
  (
    &wgpu::InstanceDescriptor
    {
      backends : wgpu::Backends::PRIMARY,
      ..Default::default()
    }
  );

  let adapter = minwgpu::helper::adapter::request_adapter
  (
    &instance,
    &wgpu::RequestAdapterOptions
    {
      power_preference : wgpu::PowerPreference::HighPerformance,
      ..Default::default()
    }
  ).expect( "Failed to retrieve an adapter" );

  minwgpu::helper::device::request_device( &adapter, &wgpu::DeviceDescriptor::default() )
  .expect( "Failed to retrieve a device" )
}

fn create_render_pipeline( device : &wgpu::Device ) -> wgpu::RenderPipeline
{
  let shader = device.create_shader_module( wgpu::include_wgsl!( "../shaders/shader.wgsl" ) );

  let render_pipeline_layout = device.create_pipeline_layout
  (
    &wgpu::PipelineLayoutDescriptor
    {
      label : Some( "hexagonal_pipeline_layout" ),
      bind_group_layouts : &[],
      push_constant_ranges : &[]
    }
  );

  device.create_render_pipeline
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
  )
}

fn create_render_target( device : &wgpu::Device, width : u32, height : u32 )
-> ( wgpu::Texture, wgpu::TextureView, wgpu::Buffer, wgpu::Extent3d )
{
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

  let bytes_per_pixel = 4;
  let buffer_size = bytes_per_pixel * width * height;
  let output_buffer_size = wgpu::BufferAddress::from( buffer_size );
  let output_buffer = device.create_buffer
  (
    &wgpu::BufferDescriptor
    {
      label : Some( "Output Buffer" ),
      size : output_buffer_size,
      usage : wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
      mapped_at_creation : false,
    }
  );

  ( texture, texture_view, output_buffer, texture_extent )
}

fn render_triangle( encoder : &mut wgpu::CommandEncoder, render_pipeline : &wgpu::RenderPipeline, texture_view : &wgpu::TextureView )
{
  let mut render_pass = encoder.begin_render_pass
  (
    &wgpu::RenderPassDescriptor
    {
      label : Some( "render_pass" ),
      color_attachments :
      &[
        Some
        (
          wgpu::RenderPassColorAttachment
          {
            view : texture_view,
            resolve_target : None,
            ops : wgpu::Operations
            {
              load : wgpu::LoadOp::Clear
              (
                wgpu::Color
                {
                  r : 0.1,
                  g : 0.2,
                  b : 0.3,
                  a : 1.0,
                }
              ),
              store : wgpu::StoreOp::Store,
            },
            depth_slice : None,
          }
        )
      ],
      depth_stencil_attachment : None,
      timestamp_writes : None,
      occlusion_query_set : None,
    }
  );
  render_pass.set_pipeline( render_pipeline );
  render_pass.draw( 0..3, 0..1 );
}

//! TODO

use wgpu::util::DeviceExt;

mod context;

fn main()
{
  run();
}

fn run()
{
  let context = context::Context::new_temp();

  let shader = context.device().create_shader_module( wgpu::include_wgsl!( "../shaders/shader.wgsl" ) );

  let width = 512;
  let height = 512;
  let texture_extent = wgpu::Extent3d
  {
    width,
    height,
    depth_or_array_layers : 1,
  };

  let texture = context.device().create_texture
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
  let output_buffer = context.device().create_buffer
  (
    &wgpu::BufferDescriptor
    {
      label : Some( "Output Buffer" ),
      size : output_buffer_size,
      usage : wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
      mapped_at_creation : false,
    }
  );

  let vertex_buffer = tiles_tools::geometry::hexagon_triangles();
  let vertex_count = ( vertex_buffer.len() / 2 ) as u32;
  let vertex_buffer = context.device().create_buffer_init
  (
    &wgpu::util::BufferInitDescriptor
    {
      label : Some( "hexagon_mesh" ),
      contents : bytemuck::cast_slice( vertex_buffer.as_slice() ),
      usage : wgpu::BufferUsages::VERTEX,
    }
  );
  let vertex_buffer_layout = wgpu::VertexBufferLayout
  {
    array_stride : ( 2 * size_of::< f32 >() ) as wgpu::BufferAddress,
    step_mode : wgpu::VertexStepMode::Vertex,
    attributes :
    &[
      wgpu::VertexAttribute
      {
        format : wgpu::VertexFormat::Float32x2,
        offset : 0,
        shader_location : 0,
      }
    ],
  };

  let uniform = Uniform
  {
    scale: 1.0,
    color : [ 1.0, 0.0, 0.0, 1.0 ],
    ..Default::default()
  };
  let uniform_buffer = context.device().create_buffer_init
  (
    &wgpu::util::BufferInitDescriptor
    {
      label : Some( "uniform_buffer" ),
      contents : bytemuck::bytes_of( &uniform ),
      usage : wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    }
  );

  let bind_group_layout = context.device().create_bind_group_layout
  (
    &wgpu::BindGroupLayoutDescriptor
    {
      label : Some("uniform_bind_group_layout"),
      entries :
      &[
        wgpu::BindGroupLayoutEntry
        {
          binding : 0,
          visibility : wgpu::ShaderStages::VERTEX_FRAGMENT,
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

  let bind_group = context.device().create_bind_group
  (
    &wgpu::BindGroupDescriptor
    {
      label : Some( "uniform_bind_group" ),
      layout : &bind_group_layout,
      entries :
      &[
        wgpu::BindGroupEntry
        {
          binding : 0,
          resource : uniform_buffer.as_entire_binding(),
        },
      ],
    }
  );

  let render_pipeline_layout = context.device().create_pipeline_layout
  (
    &wgpu::PipelineLayoutDescriptor
    {
      label : Some( "hexagonal_pipeline_layout" ),
      bind_group_layouts : &[ &bind_group_layout ],
      push_constant_ranges : &[]
    }
  );

  let render_pipeline = context.device().create_render_pipeline
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
        buffers : &[ vertex_buffer_layout ]
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
  );

  let mut encoder = context.device()
  .create_command_encoder( &wgpu::CommandEncoderDescriptor { label : Some( "encoder" ) } );

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
            view : &texture_view,
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
  render_pass.set_pipeline( &render_pipeline );
  render_pass.set_bind_group( 0, &bind_group, &[] );
  render_pass.set_vertex_buffer( 0, vertex_buffer.slice( .. ) );
  render_pass.draw( 0..vertex_count, 0..1 );
  drop( render_pass );

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

  context.queue().submit( Some( encoder.finish() ) );

  let buffer_slice = output_buffer.slice( .. );
  buffer_slice.map_async( wgpu::MapMode::Read, | _ | {} );

  context.device().poll( wgpu::PollType::Wait ).expect( "Failed to render an image" );

  let data = buffer_slice.get_mapped_range();
  image::save_buffer( "triangle.png", &data, width, height, image::ColorType::Rgba8 )
  .expect( "Failed to save image" );
}

#[ repr( C ) ]
#[ derive( Default, Clone, Copy, bytemuck::Zeroable, bytemuck::Pod ) ]
struct Uniform
{
  color : [ f32; 4 ],
  translation : [ f32; 2 ],
  scale : f32,
  _padding : [ u8; 4 ]
}

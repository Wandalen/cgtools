#![ doc = "../readme.md" ]

use tiles_tools::coordinates::{ hexagonal, pixel::Pixel, Neighbors as _ };
use hexagonal::{ Axial, Coordinate, Flat };
use minwgpu::{ buffer, context, helper };

fn main()
{
  println!( "{:?}", run() );
}

fn run() -> Result< (), minwgpu::Error >
{
  let context = context::Context::builder()
  .backends( wgpu::Backends::PRIMARY )
  .make_instance()
  .power_preference( wgpu::PowerPreference::HighPerformance )
  .request_adapter()?
  .label( "device" )
  .required_features( wgpu::Features::PUSH_CONSTANTS )
  .required_limits( wgpu::Limits { max_push_constant_size : 16, ..Default::default() } )
  .finish_context()?;

  let clear_color = wgpu::Color
  {
    r : 0.1,
    g : 0.2,
    b : 0.3,
    a : 1.0,
  };
  let hexagon_color = [ 1.0_f32, 0.0, 0.0 ];
  let outline_color = [ 0.0_f32, 0.0, 0.0 ];

  let shader = context.get_device().create_shader_module( wgpu::include_wgsl!( "../shaders/shader.wgsl" ) );

  let width = 512;
  let height = 512;
  let ( render_texture, texture_view, output_buffer, texture_extent ) =
    create_render_target( context.get_device(), width, height );

  let ( vertex_data, vertex_count, line_data, line_vertex_count ) = hexagon_mesh_data();
  let attributes = [ helper::attr( wgpu::VertexFormat::Float32x2, 0, 0 ) ];
  let vertex_buffer = buffer::vertex_buffer()
  .label( "hexagon_mesh" )
  .data( vertex_data.as_slice() )
  .array_stride( wgpu::VertexFormat::Float32x2.size() )
  .attributes( &attributes )
  .build( context.get_device() );

  let attributes = [ helper::attr( wgpu::VertexFormat::Float32x2, 0, 0 ) ];
  let line_vertex_buffer = buffer::vertex_buffer()
  .label( "hexagon_outline" )
  .data( line_data.as_slice() )
  .array_stride( wgpu::VertexFormat::Float32x2.size() )
  .attributes( &attributes )
  .build( context.get_device() );

  let ( positions, instance_count ) = hexagon_instance_positions();
  let attributes = &[ helper::attr( wgpu::VertexFormat::Float32x2, 0, 1 ) ];
  let position_buffer = buffer::vertex_buffer()
  .label( "hexagon_positions" )
  .data( positions.as_slice() )
  .array_stride( wgpu::VertexFormat::Float32x2.size() )
  .step_mode( wgpu::VertexStepMode::Instance )
  .attributes( attributes )
  .build( context.get_device() );

  let ( bind_group_layout, bind_group ) = create_uniforms( context.get_device() );

  let ( hexagon_fill_pipeline, hexagon_outline_pipeline ) = create_pipelines
  (
    &context, &shader, &vertex_buffer, &line_vertex_buffer, &position_buffer, &bind_group_layout
  );

  let mut encoder = context.get_device()
  .create_command_encoder( &wgpu::CommandEncoderDescriptor { label : Some( "encoder" ) } );

  render_scene
  (
    &mut encoder, &texture_view, clear_color, &hexagon_fill_pipeline, &hexagon_outline_pipeline, &bind_group,
    &vertex_buffer, vertex_count, &line_vertex_buffer, line_vertex_count, &position_buffer, instance_count,
    hexagon_color, outline_color,
  );

  let bytes_per_pixel = 4;
  encoder.copy_texture_to_buffer
  (
    render_texture.as_image_copy(),
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
  context.get_queue().submit( Some( encoder.finish() ) );

  let buffer_slice = output_buffer.slice( .. );
  buffer_slice.map_async( wgpu::MapMode::Read, | _ | {} );

  context.get_device().poll( wgpu::PollType::Wait{ submission_index : None, timeout : None } ).expect( "Failed to render an image" );

  let data = buffer_slice.get_mapped_range();
  image::save_buffer( "-hexagons.png", &data, width, height, image::ColorType::Rgba8 )
  .expect( "Failed to save image" );

  Ok( () )
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
  let render_texture = device.create_texture
  (
    &wgpu::TextureDescriptor
    {
      label : Some( "render_texture" ),
      size : texture_extent,
      mip_level_count : 1,
      sample_count : 1,
      dimension : wgpu::TextureDimension::D2,
      format : wgpu::TextureFormat::Rgba8UnormSrgb,
      usage : wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
      view_formats : &[],
    }
  );
  let texture_view = render_texture.create_view( &wgpu::TextureViewDescriptor::default() );

  let bytes_per_pixel = 4;
  let buffer_size = bytes_per_pixel * width * height;
  let output_buffer_size = wgpu::BufferAddress::from( buffer_size );
  let output_buffer = buffer::buffer
  (
    wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ
  )
  .label( "output_buffer" )
  .size_from_value( output_buffer_size )
  .build( device );

  ( render_texture, texture_view, output_buffer, texture_extent )
}

fn create_uniforms( device : &wgpu::Device ) -> ( wgpu::BindGroupLayout, wgpu::BindGroup )
{
  let scale_uniform = 0.25_f32;
  let uniform_buffer = buffer::buffer( wgpu::BufferUsages::UNIFORM )
  .label( "uniform_buffer" )
  .data( &[ scale_uniform ] )
  .build( device );

  let bind_group_layout = device.create_bind_group_layout
  (
    &wgpu::BindGroupLayoutDescriptor
    {
      label : Some( "uniform_bind_group_layout" ),
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

  let bind_group = device.create_bind_group
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

  ( bind_group_layout, bind_group )
}

#[ allow( clippy::too_many_arguments, reason = "each parameter is a distinct piece of GPU render-pass state (2 pipelines, their vertex/line buffers and counts, the shared bind group and instance buffer, and each pipeline's own push-constant color); grouping into a struct would add indirection without reducing call-site complexity for this single-call-site helper" ) ]
fn render_scene
(
  encoder : &mut wgpu::CommandEncoder,
  texture_view : &wgpu::TextureView,
  clear_color : wgpu::Color,
  fill_pipeline : &wgpu::RenderPipeline,
  outline_pipeline : &wgpu::RenderPipeline,
  bind_group : &wgpu::BindGroup,
  vertex_buffer : &buffer::VertexBuffer< '_ >,
  vertex_count : u32,
  line_vertex_buffer : &buffer::VertexBuffer< '_ >,
  line_vertex_count : u32,
  position_buffer : &buffer::VertexBuffer< '_ >,
  instance_count : u32,
  hexagon_color : [ f32; 3 ],
  outline_color : [ f32; 3 ],
)
{
  let render_pass_desc = &wgpu::RenderPassDescriptor
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
            load : wgpu::LoadOp::Clear( clear_color ),
            store : wgpu::StoreOp::Store,
          },
          depth_slice : None,
        }
      )
    ],
    depth_stencil_attachment : None,
    timestamp_writes : None,
    occlusion_query_set : None,
  };

  let mut render_pass = encoder.begin_render_pass( render_pass_desc );
  render_pass.set_pipeline( fill_pipeline );
  // Hexagon color
  render_pass.set_push_constants
  (
    wgpu::ShaderStages::FRAGMENT,
    0,
    asbytes::cast_slice( &hexagon_color )
  );
  render_pass.set_bind_group( 0, bind_group, &[] );
  render_pass.set_vertex_buffer( 0, vertex_buffer.as_ref().slice( .. ) );
  render_pass.set_vertex_buffer( 1, position_buffer.as_ref().slice( .. ) );
  render_pass.draw( 0..vertex_count, 0..instance_count );

  render_pass.set_pipeline( outline_pipeline );
  // Outline color
  render_pass.set_push_constants
  (
    wgpu::ShaderStages::FRAGMENT,
    0,
    asbytes::cast_slice( &outline_color )
  );
  render_pass.set_bind_group( 0, bind_group, &[] );
  render_pass.set_vertex_buffer( 0, line_vertex_buffer.as_ref().slice( .. ) );
  render_pass.set_vertex_buffer( 1, position_buffer.as_ref().slice( .. ) );
  render_pass.draw( 0..line_vertex_count, 0..instance_count );
}

fn hexagon_mesh_data() -> ( Vec< f32 >, u32, Vec< f32 >, u32 )
{
  let vertex_data = tiles_tools::geometry::hexagon_triangles();
  let vertex_count = ( vertex_data.len() / 2 ) as u32;
  let line_data = tiles_tools::geometry::hexagon_lines();
  let line_vertex_count = ( line_data.len() / 2 ) as u32;
  ( vertex_data, vertex_count, line_data, line_vertex_count )
}

fn hexagon_instance_positions() -> ( Vec< f32 >, u32 )
{
  let coord = Coordinate::< Axial, Flat >::new( 0, 0 );
  let mut hexagon_coordinates = vec![];
  hexagon_coordinates.push( coord );
  hexagon_coordinates.append( &mut coord.neighbors() );
  let instance_count = hexagon_coordinates.len() as u32;
  let positions : Vec< f32 > = hexagon_coordinates
  .into_iter()
  .flat_map( | coord | Pixel::from( coord ).data )
  .collect();
  ( positions, instance_count )
}

fn create_pipelines
(
  context : &context::Context,
  shader : &wgpu::ShaderModule,
  vertex_buffer : &buffer::VertexBuffer< '_ >,
  line_vertex_buffer : &buffer::VertexBuffer< '_ >,
  position_buffer : &buffer::VertexBuffer< '_ >,
  bind_group_layout : &wgpu::BindGroupLayout,
) -> ( wgpu::RenderPipeline, wgpu::RenderPipeline )
{
  let render_pipeline_layout = context.get_device().create_pipeline_layout
  (
    &wgpu::PipelineLayoutDescriptor
    {
      label : Some( "hexagonal_pipeline_layout" ),
      bind_group_layouts : &[ bind_group_layout ],
      push_constant_ranges : &
      [
        wgpu::PushConstantRange { stages : wgpu::ShaderStages::FRAGMENT, range : 0..16 }
      ]
    }
  );

  let hexagon_fill_pipeline = create_pipeline
  (
    context,
    shader,
    vertex_buffer,
    position_buffer,
    wgpu::PrimitiveState::default(),
    &render_pipeline_layout
  );

  let hexagon_outline_pipeline = create_pipeline
  (
    context,
    shader,
    line_vertex_buffer,
    position_buffer,
    wgpu::PrimitiveState
    {
      topology : wgpu::PrimitiveTopology::LineList,
      ..Default::default()
    },
    &render_pipeline_layout
  );

  ( hexagon_fill_pipeline, hexagon_outline_pipeline )
}

fn create_pipeline
(
  context : &context::Context,
  shader : &wgpu::ShaderModule,
  vertex_buffer : &buffer::VertexBuffer< '_ >,
  position_buffer : &buffer::VertexBuffer< '_ >,
  primitive : wgpu::PrimitiveState,
  render_pipeline_layout : &wgpu::PipelineLayout
) -> wgpu::RenderPipeline
{
  context.get_device().create_render_pipeline
  (
    &wgpu::RenderPipelineDescriptor
    {
      label : Some( "hexagonal_pipeline" ),
      layout : Some( render_pipeline_layout ),
      vertex: wgpu::VertexState
      {
        module : shader,
        entry_point : Some( "vs_main" ),
        compilation_options : wgpu::PipelineCompilationOptions::default(),
        buffers : &[ vertex_buffer.get_layout().clone(), position_buffer.get_layout().clone() ]
      },
      primitive,
      depth_stencil : None,
      multisample : wgpu::MultisampleState::default(),
      fragment : Some
      (
        wgpu::FragmentState
        {
          module : shader,
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

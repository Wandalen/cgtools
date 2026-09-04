#![ doc = "../readme.md" ]

use tiles_tools::coordinates::{ hexagonal, pixel::Pixel, Neighbors as _ };
use hexagonal::{ Axial, Coordinate, Flat };
use minwgpu::{ bind, buffer, context, helper, readback, texture };

fn main()
{
  println!( "{:?}", app_run() );
}

fn app_run() -> Result< (), minwgpu::Error >
{
  let context = context::Context::builder()
  .backends( wgpu::Backends::PRIMARY )
  .instance_make()
  .power_preference( wgpu::PowerPreference::HighPerformance )
  .adapter_request()?
  .label( "device" )
  .required_features( wgpu::Features::IMMEDIATES )
  .required_limits( wgpu::Limits { max_immediate_size : 16, ..Default::default() } )
  .context_finish()?;

  let clear_color = wgpu::Color
  {
    r : 0.1,
    g : 0.2,
    b : 0.3,
    a : 1.0,
  };
  let hexagon_color = [ 1.0_f32, 0.0, 0.0 ];
  let outline_color = [ 0.0_f32, 0.0, 0.0 ];

  let shader = context.device_get().create_shader_module( wgpu::include_wgsl!( "../shaders/shader.wgsl" ) );

  let target = texture::render_target_2d( context.device_get(), ( 512, 512 ), wgpu::TextureFormat::Rgba8UnormSrgb );

  let ( vertex_data, vertex_count, line_data, line_vertex_count ) = hexagon_mesh_data();
  let attributes = [ helper::attr( wgpu::VertexFormat::Float32x2, 0, 0 ) ];
  let vertex_buffer = buffer::vertex_buffer()
  .label( "hexagon_mesh" )
  .data( vertex_data.as_slice() )
  .array_stride( wgpu::VertexFormat::Float32x2.size() )
  .attributes( &attributes )
  .build( context.device_get() );

  let attributes = [ helper::attr( wgpu::VertexFormat::Float32x2, 0, 0 ) ];
  let line_vertex_buffer = buffer::vertex_buffer()
  .label( "hexagon_outline" )
  .data( line_data.as_slice() )
  .array_stride( wgpu::VertexFormat::Float32x2.size() )
  .attributes( &attributes )
  .build( context.device_get() );

  let ( positions, instance_count ) = hexagon_instance_positions();
  let attributes = &[ helper::attr( wgpu::VertexFormat::Float32x2, 0, 1 ) ];
  let position_buffer = buffer::vertex_buffer()
  .label( "hexagon_positions" )
  .data( positions.as_slice() )
  .array_stride( wgpu::VertexFormat::Float32x2.size() )
  .step_mode( wgpu::VertexStepMode::Instance )
  .attributes( attributes )
  .build( context.device_get() );

  let scale_uniform = 0.25_f32;
  let uniform_buffer = buffer::buffer( wgpu::BufferUsages::UNIFORM )
  .label( "uniform_buffer" )
  .data( &[ scale_uniform ] )
  .build( context.device_get() );
  let ( bind_group_layout, bind_group ) =
  bind::single_uniform( context.device_get(), &uniform_buffer, wgpu::ShaderStages::VERTEX_FRAGMENT );

  let ( hexagon_fill_pipeline, hexagon_outline_pipeline ) = pipelines_create
  (
    &context, &shader, &vertex_buffer, &line_vertex_buffer, &position_buffer, &bind_group_layout
  );

  let mut encoder = context.device_get()
  .create_command_encoder( &wgpu::CommandEncoderDescriptor { label : Some( "encoder" ) } );

  scene_render
  (
    &mut encoder, &target.view, clear_color, &hexagon_fill_pipeline, &hexagon_outline_pipeline, &bind_group,
    &vertex_buffer, vertex_count, &line_vertex_buffer, line_vertex_count, &position_buffer, instance_count,
    hexagon_color, outline_color,
  );

  context.queue_get().submit( Some( encoder.finish() ) );

  let ( pixels, ( width, height ) ) = readback::rgba8( context.device_get(), context.queue_get(), &target.texture )?;
  image::save_buffer( "-hexagons.png", &pixels, width, height, image::ColorType::Rgba8 )
  .expect( "Failed to save image" );

  Ok( () )
}

#[ allow( clippy::too_many_arguments, reason = "each parameter is a distinct piece of GPU render-pass state (2 pipelines, their vertex/line buffers and counts, the shared bind group and instance buffer, and each pipeline's own push-constant color); grouping into a struct would add indirection without reducing call-site complexity for this single-call-site helper" ) ]
fn scene_render
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
    multiview_mask : None,
  };

  let mut render_pass = encoder.begin_render_pass( render_pass_desc );
  render_pass.set_pipeline( fill_pipeline );
  // Hexagon color
  render_pass.set_immediates( 0, asbytes::cast_slice( &hexagon_color ) );
  render_pass.set_bind_group( 0, bind_group, &[] );
  render_pass.set_vertex_buffer( 0, vertex_buffer.as_ref().slice( .. ) );
  render_pass.set_vertex_buffer( 1, position_buffer.as_ref().slice( .. ) );
  render_pass.draw( 0..vertex_count, 0..instance_count );

  render_pass.set_pipeline( outline_pipeline );
  // Outline color
  render_pass.set_immediates( 0, asbytes::cast_slice( &outline_color ) );
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

fn pipelines_create
(
  context : &context::Context,
  shader : &wgpu::ShaderModule,
  vertex_buffer : &buffer::VertexBuffer< '_ >,
  line_vertex_buffer : &buffer::VertexBuffer< '_ >,
  position_buffer : &buffer::VertexBuffer< '_ >,
  bind_group_layout : &wgpu::BindGroupLayout,
) -> ( wgpu::RenderPipeline, wgpu::RenderPipeline )
{
  let render_pipeline_layout = context.device_get().create_pipeline_layout
  (
    &wgpu::PipelineLayoutDescriptor
    {
      label : Some( "hexagonal_pipeline_layout" ),
      bind_group_layouts : &[ Some( bind_group_layout ) ],
      immediate_size : 16
    }
  );

  let hexagon_fill_pipeline = pipeline_create
  (
    context,
    shader,
    vertex_buffer,
    position_buffer,
    wgpu::PrimitiveState::default(),
    &render_pipeline_layout
  );

  let hexagon_outline_pipeline = pipeline_create
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

fn pipeline_create
(
  context : &context::Context,
  shader : &wgpu::ShaderModule,
  vertex_buffer : &buffer::VertexBuffer< '_ >,
  position_buffer : &buffer::VertexBuffer< '_ >,
  primitive : wgpu::PrimitiveState,
  render_pipeline_layout : &wgpu::PipelineLayout
) -> wgpu::RenderPipeline
{
  context.device_get().create_render_pipeline
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
        buffers : &[ Some( vertex_buffer.layout_get().clone() ), Some( position_buffer.layout_get().clone() ) ]
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
      multiview_mask : None,
      cache : None
    }
  )
}

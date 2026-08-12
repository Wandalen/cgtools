#![ doc = "../readme.md" ]

#[ repr( C ) ]
#[ derive( Clone, Copy, bytemuck::Pod, bytemuck::Zeroable ) ]
struct UniformsRaw
{
  time : f32,
  seed : f32,
  node_count : i32,
  grid_density : f32
}

const SCENE_WGSL : &str = include_str!( "../shaders/scene.wgsl" );

fn main()
{
  app_run();
}

fn app_run()
{
  let context = minwgpu::context::headless_with( wgpu::Backends::VULKAN )
  .expect( "Failed to retrieve a Vulkan adapter - this backend may not be available on this machine" );
  let device = context.device_get();
  let queue = context.queue_get();

  let size = ( 512, 512 );
  let target = minwgpu::texture::render_target_2d( device, size, wgpu::TextureFormat::Rgba8UnormSrgb );

  // Single-shot render, so the uniforms are baked in at buffer creation
  // instead of updated per-frame; node_count = 4 shows off the orbiting-node
  // parameterization that the live WebGL2/WebGPU versions expose via keyboard.
  let uniforms = UniformsRaw { time : 2.0, seed : 0.0, node_count : 4, grid_density : 10.0 };
  let uniform_buffer = minwgpu::buffer::buffer( wgpu::BufferUsages::UNIFORM )
  .label( "uniform_buffer" )
  .data( &[ uniforms ] )
  .build( device );
  let ( bind_group_layout, bind_group ) =
  minwgpu::bind::single_uniform( device, &uniform_buffer, wgpu::ShaderStages::FRAGMENT );

  let render_pipeline =
  minwgpu::pipeline::fullscreen( device, SCENE_WGSL, target.texture.format(), &[ &bind_group_layout ] );

  minwgpu::pass::draw_fullscreen( device, queue, &target.view, wgpu::Color::BLACK, &render_pipeline, &[ &bind_group ] );

  let ( pixels, ( width, height ) ) = minwgpu::readback::rgba8( device, queue, &target.texture )
  .expect( "Failed to read the rendered image back" );
  image::save_buffer( "-sun_grid_lines.png", &pixels, width, height, image::ColorType::Rgba8 )
  .expect( "Failed to save image" );
}

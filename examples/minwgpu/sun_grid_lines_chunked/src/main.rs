#![ doc = "../readme.md" ]

use minwgpu_sun_grid_lines_chunked::shader_chunks;

#[ repr( C ) ]
#[ derive( Clone, Copy, bytemuck::Pod, bytemuck::Zeroable ) ]
struct UniformsRaw
{
  time : f32,
  seed : f32,
  node_count : i32,
  grid_density : f32
}

const SCENE_FRAGMENT_WGSL : &str = include_str!( "../shaders/scene_fragment.wgsl" );

fn main()
{
  app_run();
}

fn app_run()
{
  let context = minwgpu::context::headless().expect( "Failed to create a wgpu context" );
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

  // Chunks are composed by manifest-declared dependency order, not by a
  // hand-curated array position; this example's own fragment-only body is
  // then appended as the non-reusable "program" that consumes them, not a
  // chunk itself. This doesn't lose compile-time validation — include_wgsl!
  // never validated WGSL syntax either; shader compilation inside
  // pipeline::fullscreen is where that always happens, for both a
  // single-file shader and a composed one.
  let chunks_wgsl = shader_chunks::compose
  (
    &[ shader_chunks::HASH21, shader_chunks::VALUE_NOISE, shader_chunks::FBM3, shader_chunks::FULLSCREEN_TRIANGLE ]
  );
  let shader_source = format!( "{chunks_wgsl}\n\n{SCENE_FRAGMENT_WGSL}" );
  let render_pipeline =
  minwgpu::pipeline::fullscreen( device, &shader_source, target.texture.format(), &[ &bind_group_layout ] );

  minwgpu::pass::draw_fullscreen( device, queue, &target.view, wgpu::Color::BLACK, &render_pipeline, &[ &bind_group ] );

  let ( pixels, ( width, height ) ) = minwgpu::readback::rgba8( device, queue, &target.texture )
  .expect( "Failed to read the rendered image back" );
  image::save_buffer( "-sun_grid_lines_chunked.png", &pixels, width, height, image::ColorType::Rgba8 )
  .expect( "Failed to save image" );
}

#![ doc = "../readme.md" ]

const SHADER_WGSL : &str = include_str!( "../shaders/shader.wgsl" );

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

  let render_pipeline = minwgpu::pipeline::fullscreen( device, SHADER_WGSL, target.texture.format(), &[] );

  let clear = wgpu::Color { r : 0.1, g : 0.2, b : 0.3, a : 1.0 };
  minwgpu::pass::draw_fullscreen( device, queue, &target.view, clear, &render_pipeline, &[] );

  let ( pixels, ( width, height ) ) = minwgpu::readback::rgba8( device, queue, &target.texture )
  .expect( "Failed to read the rendered image back" );
  image::save_buffer( "-triangle.png", &pixels, width, height, image::ColorType::Rgba8 )
  .expect( "Failed to save image" );
}

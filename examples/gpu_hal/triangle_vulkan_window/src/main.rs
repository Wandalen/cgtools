#![ doc = "../readme.md" ]

use gpu_hal::
{
  BindGroup,
  BindGroupLayoutEntry,
  BindingResource,
  BindingType,
  Buffer,
  BufferUsage,
  ColorAttachmentDesc,
  Device,
  Error,
  IndexFormat,
  Queue,
  RenderPipeline,
  RenderPipelineDesc,
  ShaderSource,
  ShaderStages,
  StepMode,
  Surface,
  VertexAttribute,
  VertexBufferLayout,
  VertexFormat,
};
use std::time::Instant;
use winit::
{
  application::ApplicationHandler,
  event::WindowEvent,
  event_loop::{ ActiveEventLoop, ControlFlow, EventLoop },
  window::{ Window, WindowId },
};

/// Uniform-colored triangle : position passthrough vertex stage, fragment
/// stage sampling one uniform color. Identical to the shader
/// `tests/vulkan_backend_test.rs`'s offscreen `triangle_render_readback` uses,
/// so this example differs from that test in exactly one respect — where the
/// frame goes.
const WGSL : &str = "
struct Color
{
  value : vec4f
}

@group( 0 ) @binding( 0 ) var< uniform > color : Color;

@vertex
fn vs_main( @location( 0 ) position : vec2f ) -> @builtin( position ) vec4f
{
  return vec4f( position, 0.0, 1.0 );
}

@fragment
fn fs_main() -> @location( 0 ) vec4f
{
  return color.value;
}
";

const CLEAR_COLOR : [ f32 ; 4 ] = [ 0.05, 0.05, 0.12, 1.0 ];
const INITIAL_SIZE : ( u32, u32 ) = ( 800, 600 );

/// Bytes of an `f32` slice, little-endian — the layout vertex and uniform
/// buffers expect.
fn as_bytes( values : &[ f32 ] ) -> Vec< u8 >
{
  values.iter().flat_map( | v | v.to_le_bytes() ).collect()
}

/// Everything built once against the device, plus the swapchain surface it
/// presents through.
struct Renderer
{
  device : Device,
  queue : Queue,
  surface : Surface,
  pipeline : RenderPipeline,
  bind_group : BindGroup,
  uniform_buffer : Buffer,
  vertex_buffer : Buffer,
  index_buffer : Buffer,
}

impl Renderer
{
  /// Creates a Vulkan device presenting to `window`, and builds the triangle's
  /// pipeline and buffers against it.
  fn new( window : &Window ) -> Self
  {
    let size = window.inner_size();
    let ( device, queue, surface ) = Device::new_vulkan_windowed
    (
      window,
      ( size.width.max( 1 ), size.height.max( 1 ) )
    )
    .expect
    (
      "no windowed Vulkan device : this example needs a Vulkan ICD exposing \
       VK_KHR_swapchain for the current display server"
    );

    let shader = device.shader_module_create( &ShaderSource
    {
      wgsl : WGSL,
      glsl_vertex : None,
      glsl_fragment : None
    } )
    .expect( "shader module creation failed" );

    let vertices = as_bytes( &[ -0.5, -0.5, 0.5, -0.5, 0.0, 0.5 ] );
    let vertex_buffer = device.buffer_init_create( &vertices, BufferUsage::VERTEX )
    .expect( "vertex buffer creation failed" );
    let indices : Vec< u8 > = [ 0u32, 1, 2 ].iter().flat_map( | i | i.to_le_bytes() ).collect();
    let index_buffer = device.buffer_init_create( &indices, BufferUsage::INDEX )
    .expect( "index buffer creation failed" );

    let uniform_buffer = device.buffer_create( 16, BufferUsage::UNIFORM | BufferUsage::COPY_DST )
    .expect( "uniform buffer creation failed" );

    let layout = device.bind_group_layout_create
    (
      &[ BindGroupLayoutEntry
      {
        visibility : ShaderStages::FRAGMENT,
        ty : BindingType::UniformBuffer
      } ]
    )
    .expect( "bind group layout creation failed" );
    let bind_group = device.bind_group_create( &layout, &[ BindingResource::Buffer( &uniform_buffer ) ] )
    .expect( "bind group creation failed" );

    let pipeline = device.render_pipeline_create( &RenderPipelineDesc
    {
      shader : &shader,
      vertex_entry : "vs_main",
      fragment_entry : "fs_main",
      vertex_buffers : &[ VertexBufferLayout
      {
        stride : 8,
        step_mode : StepMode::Vertex,
        attributes : vec!
        [
          VertexAttribute
          {
            location : 0,
            format : VertexFormat::Float32x2,
            offset : 0
          }
        ]
      } ],
      bind_group_layouts : &[ &layout ],
      // The swapchain picks its own presentation format, so the pipeline is
      // built against whatever it chose rather than a format named here.
      color_format : surface.format(),
      depth : None,
      cull_back : false
    } )
    .expect( "pipeline creation failed" );

    Self { device, queue, surface, pipeline, bind_group, uniform_buffer, vertex_buffer, index_buffer }
  }

  /// Draws one frame, cycling the triangle's color so consecutive screenshots
  /// visibly differ — a still frame proves the swapchain presented once, a
  /// changing one proves it keeps presenting.
  fn draw( &mut self, elapsed : f32 )
  {
    let phase = elapsed * 0.8;
    let color = as_bytes
    (
      &[
        0.5 + 0.5 * phase.sin(),
        0.5 + 0.5 * ( phase + 2.094 ).sin(),
        0.5 + 0.5 * ( phase + 4.189 ).sin(),
        1.0
      ]
    );
    self.queue.buffer_write( &self.uniform_buffer, &color ).expect( "uniform write failed" );

    let view = match self.surface.current_view()
    {
      Ok( view ) => view,
      // The swapchain is out of date — repaired here rather than in
      // `current_view`, which has only `&self`. Skipping this frame is correct :
      // the next tick draws into the rebuilt chain.
      Err( Error::SurfaceNotReady ) =>
      {
        let _ = self.surface.resize( self.size() );
        return;
      }
      Err( error ) => panic!( "acquiring the next frame failed :: {error}" )
    };

    let mut encoder = self.device.command_encoder_create();
    let mut pass = encoder.render_pass_begin
    (
      &ColorAttachmentDesc { view : &view, clear : CLEAR_COLOR },
      None
    )
    .expect( "render pass failed to begin" );
    pass.pipeline_set( &self.pipeline );
    pass.bind_group_set( 0, &self.bind_group );
    pass.vertex_buffer_set( 0, &self.vertex_buffer );
    pass.index_buffer_set( &self.index_buffer, IndexFormat::Uint32 );
    pass.draw_indexed( 3 );
    pass.end();
    self.queue.submit( encoder );

    self.surface.present();
  }

  /// The swapchain's current drawable size.
  fn size( &self ) -> ( u32, u32 )
  {
    self.surface.as_vulkan_windowed()
    .expect( "this example only ever builds a windowed Vulkan surface" )
    .windowed
    .size()
  }
}

/// Window and renderer, both created on `resumed` — winit 0.30 has no window
/// before then.
struct App
{
  window : Option< Window >,
  renderer : Option< Renderer >,
  started : Instant,
}

impl ApplicationHandler for App
{
  fn resumed( &mut self, event_loop : &ActiveEventLoop )
  {
    if self.window.is_some()
    {
      return;
    }
    let attributes = Window::default_attributes()
    .with_title( "gpu_hal triangle -- Vulkan swapchain" )
    .with_inner_size( winit::dpi::LogicalSize::new( INITIAL_SIZE.0, INITIAL_SIZE.1 ) );
    let window = event_loop.create_window( attributes ).expect( "window creation failed" );

    self.renderer = Some( Renderer::new( &window ) );
    self.window = Some( window );
  }

  fn window_event( &mut self, event_loop : &ActiveEventLoop, _id : WindowId, event : WindowEvent )
  {
    match event
    {
      WindowEvent::CloseRequested => event_loop.exit(),
      WindowEvent::Resized( size ) =>
      {
        if let Some( renderer ) = self.renderer.as_mut()
        {
          // A zero dimension is a minimized window ; the surface reports that
          // as an error and keeps its existing chain, so rendering resumes on
          // its own once the window comes back.
          let _ = renderer.surface.resize( ( size.width, size.height ) );
        }
      }
      WindowEvent::RedrawRequested =>
      {
        let elapsed = self.started.elapsed().as_secs_f32();
        if let Some( renderer ) = self.renderer.as_mut()
        {
          renderer.draw( elapsed );
        }
        if let Some( window ) = self.window.as_ref()
        {
          window.request_redraw();
        }
      }
      _ => {}
    }
  }
}

fn main()
{
  let event_loop = EventLoop::new().expect( "event loop creation failed" );
  event_loop.set_control_flow( ControlFlow::Poll );
  let mut app = App { window : None, renderer : None, started : Instant::now() };
  event_loop.run_app( &mut app ).expect( "event loop failed" );
}

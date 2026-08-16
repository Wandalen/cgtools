#![ doc = "../readme.md" ]

use flecs_ecs::prelude::*;
use minwgpu::{ buffer, context, helper, surface };
use std::sync::Arc;
use winit::
{
  application::ApplicationHandler,
  event::WindowEvent,
  event_loop::{ ActiveEventLoop, ControlFlow, EventLoop },
  window::{ Window, WindowId },
};

#[ derive( Debug, Component ) ]
struct Position
{
  x : f32,
  y : f32,
}

#[ derive( Debug, Component ) ]
struct Velocity
{
  x : f32,
  y : f32,
}

#[ derive( Debug, Component ) ]
struct Radius
{
  value : f32,
}

#[ derive( Debug, Component ) ]
struct Color
{
  r : f32,
  g : f32,
  b : f32,
}

const GRAVITY : f32 = -1.4;
const ARENA_HALF : f32 = 1.0;
const RESTITUTION : f32 = 0.72;

/// Real per-frame delta time is clamped to this ceiling before stepping physics, so a stall
/// ( window drag, OS scheduling hiccup ) produces one slow-motion frame instead of a large
/// simulation jump that could tunnel a fast circle through a wall.
const MAX_DT : f32 = 0.05;

/// ( x, y, vx, vy, radius, [ r, g, b ] ).
type CircleSpec = ( f32, f32, f32, f32, f32, [ f32; 3 ] );

/// Initial state for each circle. Hardcoded rather than randomized so every run starts
/// identically.
const CIRCLES : &[ CircleSpec ] =
&[
  ( -0.80, 0.90,  0.30,  0.00, 0.10, [ 0.90, 0.20, 0.20 ] ),
  ( -0.55, 0.60, -0.20,  0.40, 0.08, [ 0.95, 0.55, 0.10 ] ),
  ( -0.30, 0.85,  0.50, -0.10, 0.12, [ 0.90, 0.85, 0.15 ] ),
  ( -0.05, 0.50,  0.10,  0.30, 0.07, [ 0.25, 0.80, 0.30 ] ),
  (  0.20, 0.88, -0.40,  0.20, 0.11, [ 0.20, 0.70, 0.85 ] ),
  (  0.45, 0.65,  0.25, -0.30, 0.09, [ 0.30, 0.40, 0.90 ] ),
  (  0.70, 0.80, -0.15,  0.00, 0.13, [ 0.65, 0.25, 0.85 ] ),
  ( -0.65, 0.30,  0.35,  0.35, 0.06, [ 0.95, 0.35, 0.65 ] ),
  (  0.10, 0.20, -0.30, -0.20, 0.14, [ 0.55, 0.85, 0.35 ] ),
  (  0.55, 0.35,  0.15,  0.25, 0.05, [ 0.85, 0.60, 0.75 ] ),
];

const QUAD_DATA : [ f32; 12 ] =
[
  -1.0, -1.0,
   1.0, -1.0,
   1.0,  1.0,
  -1.0, -1.0,
   1.0,  1.0,
  -1.0,  1.0,
];

const QUAD_ATTRIBUTES : [ wgpu::VertexAttribute; 1 ] =
[
  helper::attr( wgpu::VertexFormat::Float32x2, 0, 0 ),
];

const INSTANCE_STRIDE : wgpu::BufferAddress = 24;

const INSTANCE_ATTRIBUTES : [ wgpu::VertexAttribute; 3 ] =
[
  helper::attr( wgpu::VertexFormat::Float32x2, 0, 1 ),
  helper::attr( wgpu::VertexFormat::Float32, 8, 2 ),
  helper::attr( wgpu::VertexFormat::Float32x3, 12, 3 ),
];

fn main()
{
  let event_loop = EventLoop::new().expect( "failed to create the winit event loop" );
  // Continuous animation needs the loop to keep iterating ( and re-request redraws ) even
  // when no OS input event arrives — the default `Wait` would freeze the simulation between
  // window/keyboard/mouse events.
  event_loop.set_control_flow( ControlFlow::Poll );
  let mut app = App::default();
  event_loop.run_app( &mut app ).expect( "event loop exited with an error" );
}

/// Everything created once the window exists: the windowed GPU context, the window's own
/// presentation surface and its current configuration, the render pipeline, the static quad
/// mesh, and the flecs world driving the simulation. Held as `App::graphics : Option< _ >`
/// because `winit` only hands out a window inside `resumed`, never before.
struct Graphics
{
  window : Arc< Window >,
  context : context::Context,
  render_surface : &'static wgpu::Surface< 'static >,
  surface_config : wgpu::SurfaceConfiguration,
  pipeline : wgpu::RenderPipeline,
  quad_buffer : buffer::VertexBuffer< 'static >,
  quad_vertex_count : u32,
  world : World,
  instance_count : u32,
  clear_color : wgpu::Color,
  last_frame : std::time::Instant,
}

/// `winit`'s `ApplicationHandler` entry point. Starts with no window; `resumed` creates one
/// and everything that depends on it.
#[ derive( Default ) ]
struct App
{
  graphics : Option< Graphics >,
}

impl ApplicationHandler for App
{
  fn resumed( &mut self, event_loop : &ActiveEventLoop )
  {
    // `resumed` can fire more than once ( e.g. after `suspended` on platforms that support
    // it ); this app creates its single window once and reuses it for the rest of the run.
    if self.graphics.is_some()
    {
      return;
    }
    self.graphics = Some( graphics_init( event_loop ) );
  }

  fn window_event( &mut self, event_loop : &ActiveEventLoop, _window_id : WindowId, event : WindowEvent )
  {
    let Some( graphics ) = self.graphics.as_mut() else { return };
    match event
    {
      WindowEvent::CloseRequested => event_loop.exit(),
      WindowEvent::Resized( size ) => graphics_resize( graphics, ( size.width, size.height ) ),
      WindowEvent::RedrawRequested => frame_render( graphics ),
      _ => {}
    }
  }

  fn about_to_wait( &mut self, _event_loop : &ActiveEventLoop )
  {
    // Requesting the next redraw here ( rather than at the end of `RedrawRequested` ) is
    // `winit`'s recommended place for a continuously-animating app: it runs once per event
    // loop iteration after all pending input has been processed.
    if let Some( graphics ) = &self.graphics
    {
      graphics.window.request_redraw();
    }
  }
}

/// Creates the window, the windowed GPU context, and every render/simulation resource that
/// depends on it. Called once, from `App::resumed`.
fn graphics_init( event_loop : &ActiveEventLoop ) -> Graphics
{
  let window_attributes = Window::default_attributes()
  .with_title( "Flecs Bouncing Circles" )
  .with_inner_size( winit::dpi::LogicalSize::new( 512.0, 512.0 ) );
  let window = Arc::new( event_loop.create_window( window_attributes ).expect( "failed to create the window" ) );

  let instance = wgpu::Instance::new
  (
    wgpu::InstanceDescriptor
    {
      backends : wgpu::Backends::PRIMARY,
      ..wgpu::InstanceDescriptor::new_without_display_handle()
    }
  );
  let render_surface = instance.create_surface( window.clone() ).expect( "failed to create the window surface" );
  // Leaked deliberately: the app has exactly one window/surface pair, alive for the whole
  // process. `ContextBuilder::from` fixes every one of its lifetime parameters ( including
  // `compatible_surface`'s ) to `'static`, so a genuinely `'static` reference is the only way
  // to satisfy that signature short of re-implementing minwgpu's adapter/device request logic
  // by hand — a one-time, program-lifetime promotion, not a leak that grows over time.
  let render_surface : &'static wgpu::Surface< 'static > = Box::leak( Box::new( render_surface ) );

  let context = context::ContextBuilder::from( instance )
  .power_preference( wgpu::PowerPreference::HighPerformance )
  .compatible_surface( render_surface )
  .adapter_request()
  .expect( "failed to find a GPU adapter compatible with the window surface" )
  .context_finish()
  .expect( "failed to request a device from the selected adapter" );

  let window_size = window.inner_size();
  let surface_config = surface::surface_configure
  (
    context.device_get(),
    context.adapter_get(),
    render_surface,
    ( window_size.width.max( 1 ), window_size.height.max( 1 ) ),
  )
  .expect( "size is clamped to at least 1x1, so surface_configure cannot see a zero size here" );

  let shader = context.device_get().create_shader_module( wgpu::include_wgsl!( "../shaders/circle.wgsl" ) );

  let quad_buffer = buffer::vertex_buffer()
  .label( "circle_quad" )
  .data( &QUAD_DATA )
  .array_stride( wgpu::VertexFormat::Float32x2.size() )
  .attributes( &QUAD_ATTRIBUTES )
  .build( context.device_get() );
  let quad_vertex_count = ( QUAD_DATA.len() / 2 ) as u32;

  let world = World::new();
  systems_register( &world );
  circles_spawn( &world );
  let instance_count = CIRCLES.len() as u32;

  let bootstrap_data = instance_data_collect( &world );
  let bootstrap_buffer = instance_buffer_build( context.device_get(), &bootstrap_data );
  let pipeline = pipeline_create( &context, &shader, &quad_buffer, &bootstrap_buffer, surface_config.format );

  Graphics
  {
    window,
    context,
    render_surface,
    surface_config,
    pipeline,
    quad_buffer,
    quad_vertex_count,
    world,
    instance_count,
    clear_color : wgpu::Color { r : 0.05, g : 0.05, b : 0.08, a : 1.0 },
    last_frame : std::time::Instant::now(),
  }
}

/// Reconfigures the presentation surface for a new window size. `wgpu` requires a fresh
/// `configure` call any time the drawable size changes; skipped for a transient `0×0` size
/// ( reported while the window is minimized ), which `wgpu` would otherwise reject.
fn graphics_resize( graphics : &mut Graphics, size : ( u32, u32 ) )
{
  let ( width, height ) = size;
  if width == 0 || height == 0
  {
    return;
  }
  graphics.surface_config = surface::surface_configure
  (
    graphics.context.device_get(), graphics.context.adapter_get(), graphics.render_surface, size,
  )
  .expect( "zero sizes are filtered out by the guard above, so surface_configure cannot see one here" );
}

/// Advances the simulation by the real time elapsed since the previous frame, then renders
/// and presents the current state. Called once per `WindowEvent::RedrawRequested`.
fn frame_render( graphics : &mut Graphics )
{
  let now = std::time::Instant::now();
  let dt = now.duration_since( graphics.last_frame ).as_secs_f32().min( MAX_DT );
  graphics.last_frame = now;
  graphics.world.progress_time( dt );
  circles_collide( &graphics.world );

  let instance_data = instance_data_collect( &graphics.world );
  let instance_buffer = instance_buffer_build( graphics.context.device_get(), &instance_data );

  let surface_texture = match graphics.render_surface.get_current_texture()
  {
    wgpu::CurrentSurfaceTexture::Success( texture )
    | wgpu::CurrentSurfaceTexture::Suboptimal( texture ) => texture,
    // Transient states around resize/minimize/occlusion races that the `Resized` handler
    // already reconfigures for; skipping this frame and re-acquiring on the next one recovers
    // cleanly. `Validation` indicates a programming error rather than a transient condition.
    wgpu::CurrentSurfaceTexture::Timeout
    | wgpu::CurrentSurfaceTexture::Occluded
    | wgpu::CurrentSurfaceTexture::Outdated
    | wgpu::CurrentSurfaceTexture::Lost => return,
    wgpu::CurrentSurfaceTexture::Validation =>
    {
      panic!( "surface validation error while acquiring the next swapchain texture" )
    }
  };
  let view = surface_texture.texture.create_view( &wgpu::TextureViewDescriptor::default() );

  let mut encoder = graphics.context.device_get()
  .create_command_encoder( &wgpu::CommandEncoderDescriptor { label : Some( "encoder" ) } );

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
            view : &view,
            resolve_target : None,
            ops : wgpu::Operations
            {
              load : wgpu::LoadOp::Clear( graphics.clear_color ),
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
    render_pass.set_pipeline( &graphics.pipeline );
    render_pass.set_vertex_buffer( 0, graphics.quad_buffer.as_ref().slice( .. ) );
    render_pass.set_vertex_buffer( 1, instance_buffer.as_ref().slice( .. ) );
    render_pass.draw( 0..graphics.quad_vertex_count, 0..graphics.instance_count );
  }

  graphics.context.queue_get().submit( Some( encoder.finish() ) );
  graphics.window.pre_present_notify();
  graphics.context.queue_get().present( surface_texture );
}

/// Registers the two per-frame systems. `GravityIntegrate` accelerates and moves every
/// circle; `WallBounce` reflects velocity when a circle's edge crosses the arena bounds.
/// Both run every call to `world.progress_time( dt )`, in this registration order.
fn systems_register( world : &World )
{
  world
  .system_named::< ( &mut Position, &mut Velocity ) >( "GravityIntegrate" )
  .each_iter( | it, _index, ( pos, vel ) |
  {
    let dt = it.delta_time();
    vel.y += GRAVITY * dt;
    pos.x += vel.x * dt;
    pos.y += vel.y * dt;
  } );

  world
  .system_named::< ( &mut Position, &mut Velocity, &Radius ) >( "WallBounce" )
  .each( | ( pos, vel, radius ) |
  {
    let r = radius.value;
    if pos.x - r < -ARENA_HALF
    {
      pos.x = -ARENA_HALF + r;
      vel.x = vel.x.abs() * RESTITUTION;
    }
    if pos.x + r > ARENA_HALF
    {
      pos.x = ARENA_HALF - r;
      vel.x = -vel.x.abs() * RESTITUTION;
    }
    if pos.y - r < -ARENA_HALF
    {
      pos.y = -ARENA_HALF + r;
      vel.y = vel.y.abs() * RESTITUTION;
    }
    if pos.y + r > ARENA_HALF
    {
      pos.y = ARENA_HALF - r;
      vel.y = -vel.y.abs() * RESTITUTION;
    }
  } );
}

/// Resolves overlaps left after `GravityIntegrate` and `WallBounce` have moved and wall-clamped
/// every circle this frame: for each overlapping pair, pushes the two circles apart along the
/// line between their centers and applies an elastic velocity response scaled by `RESTITUTION`,
/// treating every circle as equal mass since no mass component exists. Called once per frame
/// from `frame_render`, immediately after `world.progress_time( dt )`.
///
/// A plain function rather than a third `system_named` registration: the O( n² ) pairwise scan
/// needs every circle's state gathered before any of it is resolved, which `each`/`each_iter`'s
/// one-row-at-a-time callback can't express. Collected into a local `Vec` first via
/// `each_entity` ( the `EntityView` it hands the callback is only valid for that single call, so
/// only its underlying `Entity` — a plain `Copy` id with no such restriction — is kept ),
/// resolved locally, then written back through `World::entity_from_id`.
fn circles_collide( world : &World )
{
  struct Circle
  {
    entity : Entity,
    x : f32,
    y : f32,
    vx : f32,
    vy : f32,
    r : f32,
  }

  let mut circles = Vec::new();
  world
  .new_query::< ( &Position, &Velocity, &Radius ) >()
  .each_entity( | entity, ( pos, vel, radius ) |
  {
    circles.push( Circle { entity : *entity, x : pos.x, y : pos.y, vx : vel.x, vy : vel.y, r : radius.value } );
  } );

  for i in 0..circles.len()
  {
    for j in ( i + 1 )..circles.len()
    {
      let dx = circles[ j ].x - circles[ i ].x;
      let dy = circles[ j ].y - circles[ i ].y;
      let min_dist = circles[ i ].r + circles[ j ].r;
      let dist_sq = dx * dx + dy * dy;
      if dist_sq <= f32::EPSILON || dist_sq >= min_dist * min_dist
      {
        continue;
      }

      let dist = dist_sq.sqrt();
      let ( nx, ny ) = ( dx / dist, dy / dist );

      let penetration = min_dist - dist;
      circles[ i ].x -= nx * penetration * 0.5;
      circles[ i ].y -= ny * penetration * 0.5;
      circles[ j ].x += nx * penetration * 0.5;
      circles[ j ].y += ny * penetration * 0.5;

      let rel_vel_n = ( circles[ j ].vx - circles[ i ].vx ) * nx + ( circles[ j ].vy - circles[ i ].vy ) * ny;
      if rel_vel_n < 0.0
      {
        let impulse = -( 1.0 + RESTITUTION ) * rel_vel_n * 0.5;
        circles[ i ].vx -= impulse * nx;
        circles[ i ].vy -= impulse * ny;
        circles[ j ].vx += impulse * nx;
        circles[ j ].vy += impulse * ny;
      }
    }
  }

  for circle in circles
  {
    world.entity_from_id( circle.entity )
    .set( Position { x : circle.x, y : circle.y } )
    .set( Velocity { x : circle.vx, y : circle.vy } );
  }
}

fn circles_spawn( world : &World )
{
  for &( x, y, vx, vy, radius, color ) in CIRCLES
  {
    world.entity()
    .set( Position { x, y } )
    .set( Velocity { x : vx, y : vy } )
    .set( Radius { value : radius } )
    .set( Color { r : color[ 0 ], g : color[ 1 ], b : color[ 2 ] } );
  }
}

/// Reads every circle's current state into a flat, interleaved instance buffer:
/// `[ center.x, center.y, radius, color.r, color.g, color.b ]` per circle.
fn instance_data_collect( world : &World ) -> Vec< f32 >
{
  let mut data = Vec::new();
  world
  .new_query::< ( &Position, &Radius, &Color ) >()
  .each( | ( pos, radius, color ) |
  {
    data.push( pos.x );
    data.push( pos.y );
    data.push( radius.value );
    data.push( color.r );
    data.push( color.g );
    data.push( color.b );
  } );
  data
}

fn instance_buffer_build< 'a >( device : &wgpu::Device, data : &'a [ f32 ] ) -> buffer::VertexBuffer< 'a >
{
  buffer::vertex_buffer()
  .label( "circle_instances" )
  .data( data )
  .array_stride( INSTANCE_STRIDE )
  .step_mode( wgpu::VertexStepMode::Instance )
  .attributes( &INSTANCE_ATTRIBUTES )
  .build( device )
}

fn pipeline_create
(
  context : &context::Context,
  shader : &wgpu::ShaderModule,
  quad_buffer : &buffer::VertexBuffer< '_ >,
  instance_buffer : &buffer::VertexBuffer< '_ >,
  surface_format : wgpu::TextureFormat,
) -> wgpu::RenderPipeline
{
  let layout = context.device_get().create_pipeline_layout
  (
    &wgpu::PipelineLayoutDescriptor
    {
      label : Some( "circles_pipeline_layout" ),
      bind_group_layouts : &[],
      immediate_size : 0,
    }
  );

  context.device_get().create_render_pipeline
  (
    &wgpu::RenderPipelineDescriptor
    {
      label : Some( "circles_pipeline" ),
      layout : Some( &layout ),
      vertex : wgpu::VertexState
      {
        module : shader,
        entry_point : Some( "vs_main" ),
        compilation_options : wgpu::PipelineCompilationOptions::default(),
        buffers : &[ Some( quad_buffer.layout_get().clone() ), Some( instance_buffer.layout_get().clone() ) ],
      },
      primitive : wgpu::PrimitiveState::default(),
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
                format : surface_format,
                blend : Some( wgpu::BlendState::ALPHA_BLENDING ),
                write_mask : wgpu::ColorWrites::ALL,
              }
            )
          ],
        }
      ),
      multiview_mask : None,
      cache : None,
    }
  )
}

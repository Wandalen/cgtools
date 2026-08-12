//! Canonical `gpu_hal` opaque PBR scene on the `renderer::webgpu` path.
//!
//! A metallic-roughness sphere grid over a ground plane, lit by one
//! directional, one point and one spot light — HDR opaque pass + ACES tone
//! mapping, written once against `gpu_hal` and running on either backend.
//! WebGPU is picked when the browser exposes `navigator.gpu`; append
//! `?webgl` to the URL to force the WebGL2 fallback. The page title names
//! the active backend.
//!
//! This example only works on WebAssembly ( wasm32 ) targets.

#[ cfg( target_arch = "wasm32" ) ]
use minwebgpu as gl;

#[ cfg( target_arch = "wasm32" ) ]
mod app
{
  use minwebgpu as gl;
  use renderer::webgpu::{ Frame, Geometry, GpuContext, Lights, PbrMaterial, RenderItem, WebGpuRenderer };
  use gpu_hal::{ DepthRange, Error };

  /// CPU-side mesh attribute arrays, in `Geometry::new` argument order.
  struct MeshData
  {
    positions : Vec< f32 >,
    normals : Vec< f32 >,
    uvs : Vec< f32 >,
    colors : Vec< f32 >,
    indices : Vec< u32 >
  }

  /// Indexed UV sphere: positions, normals, uvs, white vertex colors,
  /// CCW-outward triangles.
  fn sphere_mesh( radius : f32, sectors : u32, stacks : u32 ) -> MeshData
  {
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut colors = Vec::new();
    let mut indices = Vec::new();

    for stack in 0 ..= stacks
    {
      let phi = core::f32::consts::PI * stack as f32 / stacks as f32;
      for sector in 0 ..= sectors
      {
        let theta = 2.0 * core::f32::consts::PI * sector as f32 / sectors as f32;
        let normal = [ phi.sin() * theta.cos(), phi.cos(), phi.sin() * theta.sin() ];
        positions.extend_from_slice( &[ radius * normal[ 0 ], radius * normal[ 1 ], radius * normal[ 2 ] ] );
        normals.extend_from_slice( &normal );
        uvs.extend_from_slice( &[ sector as f32 / sectors as f32, stack as f32 / stacks as f32 ] );
        colors.extend_from_slice( &[ 1.0, 1.0, 1.0, 1.0 ] );
      }
    }

    for stack in 0 .. stacks
    {
      for sector in 0 .. sectors
      {
        let a = stack * ( sectors + 1 ) + sector;
        let b = a + sectors + 1;
        indices.extend_from_slice( &[ a, a + 1, b, a + 1, b + 1, b ] );
      }
    }

    MeshData { positions, normals, uvs, colors, indices }
  }

  /// Indexed XZ ground quad centered at the origin, +Y normal, CCW from above.
  fn plane_mesh( half : f32 ) -> MeshData
  {
    let positions = vec!
    [
      -half, 0.0, -half,
      -half, 0.0, half,
      half, 0.0, half,
      half, 0.0, -half
    ];
    let normals = vec![ 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0 ];
    let uvs = vec![ 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0, 0.0 ];
    let colors = vec![ 1.0; 16 ];
    let indices = vec![ 0, 1, 2, 0, 2, 3 ];
    MeshData { positions, normals, uvs, colors, indices }
  }

  /// Builds the draw list: a 5x2 sphere grid ( metallic and dielectric rows,
  /// roughness rising left to right ) over a ground plane.
  fn items_build( context : &GpuContext, renderer : &WebGpuRenderer ) -> Result< Vec< RenderItem >, Error >
  {
    let mut items = Vec::new();

    let rows : [ ( f32, [ f32; 4 ], f32 ); 2 ] =
    [
      ( 1.0, [ 1.0, 0.766, 0.336, 1.0 ], -1.0 ), // gold, metallic row
      ( 0.0, [ 0.7, 0.1, 0.08, 1.0 ], 1.0 ) // red, dielectric row
    ];
    for ( metallic, base_color, z ) in rows
    {
      for i in 0 .. 5
      {
        let mesh = sphere_mesh( 0.6, 32, 16 );
        let geometry = Geometry::new( &context.device, &mesh.positions, &mesh.normals, &mesh.uvs, &mesh.colors, Some( mesh.indices ) )?;
        let material = PbrMaterial
        {
          base_color_factor : base_color,
          metallic_factor : metallic,
          roughness_factor : 0.05 + i as f32 * 0.225,
          ..PbrMaterial::new()
        };
        let binding = renderer.create_material_binding( context, &material )?;
        let position = gl::math::F32x3::from( [ ( i as f32 - 2.0 ) * 1.7, 0.62, z ] );
        let world = gl::math::mat3x3h::translation( position );
        items.push( renderer.create_item( context, geometry, binding, world )? );
      }
    }

    let mesh = plane_mesh( 9.0 );
    let geometry = Geometry::new( &context.device, &mesh.positions, &mesh.normals, &mesh.uvs, &mesh.colors, Some( mesh.indices ) )?;
    let material = PbrMaterial
    {
      base_color_factor : [ 0.55, 0.57, 0.6, 1.0 ],
      metallic_factor : 0.0,
      roughness_factor : 0.85,
      ..PbrMaterial::new()
    };
    let binding = renderer.create_material_binding( context, &material )?;
    let world = gl::math::mat3x3h::translation( gl::math::F32x3::from( [ 0.0, 0.0, 0.0 ] ) );
    items.push( renderer.create_item( context, geometry, binding, world )? );

    Ok( items )
  }

  /// One light of each supported kind: warm sun, cool point fill, white spot.
  fn lights_build() -> Lights
  {
    let mut lights = Lights::new();
    assert!( lights.push_direct( [ 1.0, 2.0, 1.0 ], [ 1.0, 0.96, 0.88 ], 3.0 ) );
    assert!( lights.push_point( [ -4.0, 3.0, 3.0 ], [ 0.3, 0.5, 1.0 ], 30.0, 25.0 ) );
    assert!( lights.push_spot( [ 0.0, 6.0, 5.0 ], [ 0.0, -1.0, -0.8 ], [ 1.0, 1.0, 1.0 ], 60.0, 30.0, 0.35, 0.55 ) );
    lights
  }

  /// Sets up the chosen backend and runs the render loop.
  pub async fn app_run() -> Result< (), Error >
  {
    gl::browser::setup( gl::browser::Config::default() );
    let canvas = gl::canvas::retrieve_or_make().map_err( gl::WebGPUError::from )?;

    let window = gl::web_sys::window().unwrap();
    let document = window.document().unwrap();
    let force_webgl = document.url().is_ok_and( | url | url.contains( "webgl" ) );
    let has_webgpu = !AsRef::< gl::JsValue >::as_ref( &window.navigator().gpu() ).is_undefined();

    let ( context, backend ) = if has_webgpu && !force_webgl
    {
      ( GpuContext::new_webgpu( &canvas ).await?, "WebGPU" )
    }
    else
    {
      ( GpuContext::new_webgl( &canvas )?, "WebGL2" )
    };
    document.set_title( &format!( "renderer PBR scene — {backend}" ) );

    let renderer = WebGpuRenderer::new( &context )?;
    let items = items_build( &context, &renderer )?;
    let lights = lights_build();

    let aspect = canvas.width() as f32 / canvas.height() as f32;
    let fovy = 40f32.to_radians();
    // The projection's clip-space depth range must match the backend's.
    let projection_matrix = match context.device.depth_range()
    {
      DepthRange::ZeroToOne => gl::math::mat3x3h::perspective_rh( fovy, aspect, 0.1, 100.0 ),
      DepthRange::NegOneToOne => gl::math::mat3x3h::perspective_rh_gl( fovy, aspect, 0.1, 100.0 )
    };

    let update_and_draw = move | t : f64 |
    {
      let angle = 0.25 * ( t / 1000.0 ) as f32;
      let eye = gl::math::F32x3::from( [ 8.0 * angle.sin(), 3.6, 8.0 * angle.cos() ] );
      let center = gl::math::F32x3::from( [ 0.0, 0.5, 0.0 ] );
      let frame = Frame
      {
        view_matrix : gl::math::mat3x3h::look_at_rh( eye, center, gl::math::F32x3::Y ),
        projection_matrix,
        eye,
        exposure : 0.0
      };
      renderer.render( &context, &frame, &lights, &items ).unwrap();
      true
    };

    gl::exec_loop::run( update_and_draw );

    Ok( () )
  }
}

#[ cfg( target_arch = "wasm32" ) ]
fn main()
{
  gl::spawn_local( async move { app::app_run().await.unwrap() } );
}

#[ cfg( not( target_arch = "wasm32" ) ) ]
fn main()
{
  println!( "This example targets wasm32-unknown-unknown; build it with Trunk for the browser." );
}

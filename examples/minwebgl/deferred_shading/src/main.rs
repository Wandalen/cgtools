//! This example demonstrates deferred shading with light volumes in WebGL.
//!
//! Deferred shading separates the rendering process into two passes:
//! 1. Geometry Pass: Renders scene geometry and stores properties (position, normal, color)
//!    into multiple textures called a G-buffer.
//! 2. Lighting Pass: Uses the G-buffer textures to calculate lighting for each pixel,
//!    without needing to re-render the geometry.
//!
//! Light volumes further optimize the lighting pass by only calculating lighting
//! for pixels within the bounding volume of each light source.

#![ allow( clippy::std_instead_of_core ) ]
#![ allow( clippy::implicit_return ) ]
#![ allow( clippy::default_trait_access ) ]
#![ allow( clippy::too_many_lines ) ]
#![ allow( clippy::cast_possible_truncation ) ]
#![ allow( clippy::cast_precision_loss ) ]
#![ allow( clippy::needless_borrow ) ]
#![ allow( clippy::float_cmp ) ]
#![ allow( clippy::many_single_char_names ) ]
#![ allow( clippy::indexing_slicing ) ]
#![ allow( clippy::cast_sign_loss ) ]
#![ allow( clippy::similar_names ) ]
#![ allow( clippy::module_name_repetitions ) ]
#![ allow( clippy::wildcard_imports ) ]
#![ allow( clippy::min_ident_chars ) ]
#![ allow( clippy::uninlined_format_args ) ]
#![ allow( clippy::cast_possible_wrap ) ]
#![ allow( clippy::type_complexity ) ]

mod elliptical_orbit;
mod lil_gui;
mod types;
mod framebuffer;
mod geometry;
mod light;
mod shader;

use minwebgl as gl;
use renderer::webgl::loaders::gltf;
use types::GuiParams;
use gl::
{
  GL,
  F32x3,
  math::d2::mat3x3h,
  AsBytes as _,
  JsCast as _,
};
use web_sys::HtmlCanvasElement;

fn main()
{
  gl::browser::setup( Default::default() );
  gl::spawn_local( async move { gl::info!( "{:?}", run().await ) } );
}

async fn run() -> Result< (), gl::WebglError >
{
  // Get window and document
  let window = web_sys::window().unwrap();
  let document =  window.document().unwrap();

  // Calculate canvas size based on window size and device pixel ratio
  let fwidth = window.inner_width().unwrap().as_f64().unwrap();
  let fheight = window.inner_height().unwrap().as_f64().unwrap();
  let dpr = window.device_pixel_ratio();
  let width = ( fwidth * dpr ) as i32;
  let height = ( fheight * dpr ) as i32;
  let aspect = width as f32 / height as f32;

  // Retrieve or create the WebGL context
  let gl = gl::context::retrieve_or_make().expect( "Failed to retrieve WebGl context" );
  // Enable the EXT_color_buffer_float extension for floating-point textures
  let ext = gl.get_extension( "EXT_color_buffer_float" ).unwrap().unwrap();
  gl::info!( "{}", ext.to_string() );
  // Get the canvas element and set its size
  let canvas = gl.canvas().unwrap().dyn_into::< HtmlCanvasElement >().unwrap();
  canvas.set_width( width as u32 );
  canvas.set_height( height as u32 );

  // Load the Sponza GLTF model
  let sponza = gltf::load( &document, "sponza.glb", &gl ).await?;
  gl.bind_vertex_array( None );
  let sphere = gltf::load( &document, "sphere.glb", &gl ).await?;

  // Update world matrices and calculate bounding box
  sponza.scenes[ 0 ].borrow_mut().update_world_matrix();
  let scene_bounding_box = sponza.scenes[ 0 ].borrow().bounding_box();
  gl::info!( "Scene bounding box: {:?}", scene_bounding_box );

  // Configure initial WebGL state
  gl.viewport( 0, 0, width, height );
  gl.enable( GL::DEPTH_TEST );
  gl.enable( GL::CULL_FACE );
  gl.cull_face( GL::BACK );
  gl.clear_color( 0.0, 0.0, 0.0, 1.0 );
  gl.blend_func( gl::ONE, gl::ONE );

  // Load all shaders
  let shaders = shader::load_shaders( &gl )?;

  // Setup scene transformations
  let rotation = mat3x3h::rot( 10.0f32.to_radians(), 0.0, 0.0 )
  * mat3x3h::rot( 0.0, 90.0f32.to_radians(), 0.0 );
  let scale = 0.1;
  let scene_transform = mat3x3h::translation( [ 0.0f32, -40.0, -95.0 ] )
  * rotation
  * mat3x3h::scale( [ scale, scale, scale ] );

  // Calculate transformed scene center for camera look-at
  let local_center = scene_bounding_box.center();
  let center_4d = F32x3::new( local_center.x(), local_center.y(), local_center.z() );
  // Transform the center to world space using scene_transform
  let center_homogeneous = gl::math::F32x4::new( center_4d.x(), center_4d.y(), center_4d.z(), 1.0 );
  let transformed_center_4d = scene_transform * center_homogeneous;
  let scene_center = F32x3::new
  (
    transformed_center_4d.x(),
    transformed_center_4d.y(),
    transformed_center_4d.z()
  );
  gl::info!( "Scene center (world space): {:?}", scene_center );

  // Calculate camera distance based on bounding box size
  let diagonal = ( scene_bounding_box.max - scene_bounding_box.min ).mag() * scale;
  let camera_distance = diagonal * 1.5; // Position camera at 1.5x the diagonal distance

  // Setup camera with interactive controls
  let camera_position = F32x3::new
  (
    scene_center.x(),
    scene_center.y(),
    scene_center.z() + camera_distance
  );

  let mut camera = renderer::webgl::Camera::new
  (
    camera_position,                 // Camera position (in front of scene center)
    [ 0.0, 1.0, 0.0 ].into(),       // Up vector
    scene_center,                    // Look at target (actual center of scene)
    aspect,
    60.0_f32.to_radians(),          // Field of view
    0.1,                             // Near plane
    1000.0                           // Far plane
  );
  camera.set_window_size( [ width as f32, height as f32 ].into() );
  camera.bind_controls( &canvas );

  // Create framebuffers
  let fb = framebuffer::create_framebuffers( &gl, width, height );

  // Setup GUI parameters
  let params = GuiParams
  {
    light_count : 200,
    light_color : [ 0.5, 0.5, 0.5 ],
    min_radius : 11.0,
    max_radius : 19.0,
    intensity : 1.0,
  };
  let params_obj = serde_wasm_bindgen::to_value( &params ).unwrap();

  // Create lil-gui interface
  let gui = lil_gui::new_gui();
  lil_gui::add( &gui, &params_obj, "light_count", Some( 1.0 ), Some( 5000.0 ), Some( 1.0 ) );
  lil_gui::add_color( &gui, &params_obj, "light_color" );
  lil_gui::add( &gui, &params_obj, "min_radius", Some( 1.0 ), Some( 50.0 ), Some( 0.1 ) );
  lil_gui::add( &gui, &params_obj, "max_radius", Some( 1.0 ), Some( 100.0 ), Some( 0.1 ) );
  lil_gui::add( &gui, &params_obj, "intensity", Some( 0.1 ), Some( 5.0 ), Some( 0.1 ) );

  // Create light system
  let max_light_count = 5000;
  let mut lights = light::create_light_system( &gl, max_light_count, params.min_radius, params.max_radius )?;

  // Function to generate radii (needs max_count captured)
  let generate_radii = move | min : f32, max : f32 | -> Vec< f32 >
  {
    let mut radii = ( 0..max_light_count )
      .map( | _ | rand::random_range( min..=max ) )
      .collect::< Vec< _ > >();
    radii[ 0 ] = 100.0;
    radii
  };

  // Create geometry
  let geom = geometry::create_geometry( &gl, &sphere, &lights.translation_buffer, &lights.radius_buffer )?;

  // Get UI elements
  let fps_counter = document.get_element_by_id( "fps-counter" ).unwrap();

  // Variables for FPS calculation
  let mut last_time = 0.0;
  let mut fps = 0;

  // The main update and render loop
  let update = move | time_millis |
  {
    let current_time = ( time_millis / 1000.0 ) as f32;
    // Update fps text when a whole second elapsed
    if current_time as u32 > last_time as u32
    {
      fps_counter.set_text_content( Some( &format!( "fps: {}", fps ) ) );
      fps = 0;
    }
    last_time = current_time;
    fps += 1;

    // Read GUI parameters
    let mut params : GuiParams = serde_wasm_bindgen::from_value( params_obj.clone() ).unwrap();
    let light_count = params.light_count;

    // Validate radius range: ensure min_radius <= max_radius
    if params.min_radius > params.max_radius
    {
      params.min_radius = params.max_radius;
    }

    // Check if radius range has changed and regenerate radii if needed
    let mut prev_range = lights.prev_radius_range.borrow_mut();
    if prev_range.0 != params.min_radius || prev_range.1 != params.max_radius
    {
      // Regenerate light radii with new range (now guaranteed min <= max)
      let new_radii = generate_radii( params.min_radius, params.max_radius );
      *lights.radii.borrow_mut() = new_radii;
      // Update the radius buffer
      gl.bind_buffer( GL::ARRAY_BUFFER, Some( &lights.radius_buffer ) );
      gl.buffer_sub_data_with_i32_and_u8_array
      (
        GL::ARRAY_BUFFER,
        0,
        lights.radii.borrow().as_bytes()
      );
      // Update prev range
      *prev_range = ( params.min_radius, params.max_radius );
    }
    drop( prev_range );

    // Update light positions based on their elliptical orbits
    lights.orbits[ 1..light_count ].iter().zip( lights.offsets[ 1..light_count ].iter() ).enumerate()
    .for_each
    (
      | ( i, ( orbit, offset ) ) |
      lights.translations[ i + 1 ] = orbit.position_at_angle( 0.3 * current_time + *offset ).0
    );

    // Update the translation buffer with the new light positions
    gl.bind_buffer( GL::ARRAY_BUFFER, Some( &lights.translation_buffer ) );
    gl.buffer_sub_data_with_i32_and_u8_array_and_src_offset
    (
      GL::ARRAY_BUFFER,
      size_of::< [ f32; 3 ] >() as i32,
      lights.translations[ 1..light_count ].as_bytes(),
      0
    );

    // Get view and projection matrices from the camera
    let view = camera.get_view_matrix();
    let projection = camera.get_projection_matrix();
    let scene_mvp = projection * view * scene_transform;

    // --- Geometry Pass ---
    gl.enable( GL::DEPTH_TEST ); // Enable depth test
    gl.depth_mask( true ); // Enable depth writing
    gl.depth_func( GL::LESS ); // Set depth function

    // Bind the G-buffer framebuffer
    gl.bind_framebuffer( GL::FRAMEBUFFER, fb.gbuffer.as_ref() );
    // Attach the depth buffer to the G-buffer framebuffer
    gl.framebuffer_renderbuffer
    (
      GL::FRAMEBUFFER,
      GL::DEPTH_ATTACHMENT,
      GL::RENDERBUFFER,
      fb.depthbuffer.as_ref()
    );
    // Specify which color attachments to draw to
    gl::drawbuffers::drawbuffers( &gl, &[ 0, 1, 2 ] );
    gl.clear( GL::DEPTH_BUFFER_BIT );

    // Activate the object shader and set uniforms
    shaders.object.activate();
    shaders.object.uniform_matrix_upload( "u_model", scene_transform.raw_slice(), true );
    shaders.object.uniform_matrix_upload( "u_rotation", rotation.raw_slice(), true );
    shaders.object.uniform_matrix_upload( "u_mvp", scene_mvp.raw_slice(), true );

    // Draw the Sponza model
    for mesh in &sponza.meshes
    {
      for primitive in &mesh.borrow().primitives
      {
        let primitive = primitive.borrow();
        let material = primitive.material.borrow();
        // Bind the base color texture if it exists
        let Some( base_color ) = material.base_color_texture.as_ref() else
        {
          continue;
        };
        primitive.geometry.borrow().bind( &gl ); // Bind the primitive's geometry
        gl.active_texture( GL::TEXTURE0 ); // Activate texture unit 0
        base_color.bind( &gl ); // Bind the base color texture to texture unit 0
        primitive.draw( &gl ); // Draw the primitive
      }
    }

    // --- Lighting Pass ---
    // Draw back faces of light volumes and clip fragments that are behind the back face
    gl.cull_face( GL::FRONT ); // Cull front faces
    gl.depth_func( GL::GEQUAL ); // Change depth function to GEQUAL
    // Blending is needed when a fragment is affected by several lights (additive blending)
    gl.enable( gl::BLEND );
    gl.depth_mask( false ); // Disable depth writing

    // Bind the offscreen framebuffer for the lighting result
    gl.bind_framebuffer( GL::FRAMEBUFFER, fb.offscreen_buffer.as_ref() );
    gl.framebuffer_renderbuffer
    (
      GL::FRAMEBUFFER,
      GL::DEPTH_ATTACHMENT,
      GL::RENDERBUFFER,
      fb.depthbuffer.as_ref()
    );
    gl::drawbuffers::drawbuffers( &gl, &[ 0 ] );
    gl.clear( gl::COLOR_BUFFER_BIT );

    // Bind the G-buffer textures to texture units
    gl.active_texture( GL::TEXTURE0 );
    gl.bind_texture( GL::TEXTURE_2D, fb.position_gbuffer.as_ref() );
    gl.active_texture( GL::TEXTURE1 );
    gl.bind_texture( GL::TEXTURE_2D, fb.normal_gbuffer.as_ref() );
    gl.active_texture( GL::TEXTURE2 );
    gl.bind_texture( GL::TEXTURE_2D, fb.color_gbuffer.as_ref() );

    // Bind the light volume geometry and activate the light shader
    geom.light_volume.bind( &gl );
    shaders.light.activate();
    shaders.light.uniform_matrix_upload( "u_mvp", ( projection * view ).raw_slice(), true );
    shaders.light.uniform_upload( "u_camera_position", camera.get_eye().as_slice() );
    shaders.light.uniform_upload( "u_screen_size", [ width as f32, height as f32 ].as_slice() );
    shaders.light.uniform_upload( "u_positions", &0 );
    shaders.light.uniform_upload( "u_normals", &1 );
    shaders.light.uniform_upload( "u_colors", &2 );

    let light_color_with_intensity =
    [
      params.light_color[ 0 ] * params.intensity,
      params.light_color[ 1 ] * params.intensity,
      params.light_color[ 2 ] * params.intensity
    ];
    gl.vertex_attrib3f( 3, light_color_with_intensity[ 0 ], light_color_with_intensity[ 1 ], light_color_with_intensity[ 2 ] );

    // Draw the light volumes instanced
    geom.light_volume.draw_instanced( &gl, light_count as i32 );

    // --- Light Source Visualization Pass ---
    // Draw small spheres at light positions to visualize light sources
    // This happens before the screen pass so spheres are properly occluded by geometry
    gl.disable( gl::BLEND ); // Disable blending for opaque sphere rendering
    gl.enable( GL::DEPTH_TEST );
    gl.depth_func( GL::LESS );
    gl.depth_mask( true );
    gl.cull_face( GL::BACK ); // Reset culling face

    // Keep offscreen_buffer bound and attach depth buffer for occlusion
    gl.bind_framebuffer( GL::FRAMEBUFFER, fb.offscreen_buffer.as_ref() );
    gl.framebuffer_renderbuffer
    (
      GL::FRAMEBUFFER,
      GL::DEPTH_ATTACHMENT,
      GL::RENDERBUFFER,
      fb.depthbuffer.as_ref()
    );

    // Bind the sphere geometry
    geom.light_sphere.borrow().bind( &gl );
    shaders.light_sphere.activate();
    shaders.light_sphere.uniform_matrix_upload( "u_view_projection", ( projection * view ).raw_slice(), true );
    shaders.light_sphere.uniform_upload( "u_scale", &0.2_f32 );
    shaders.light_sphere.uniform_upload( "u_color", params.light_color.as_slice() );

    // Draw the spheres instanced at each light position
    geom.light_sphere.borrow().draw_instanced( &gl, light_count as i32 );

    // Disable depth test for the final screen pass
    gl.disable( GL::DEPTH_TEST );

    // --- Final Screen Pass ---
    gl.bind_framebuffer( GL::FRAMEBUFFER, None );
    gl.active_texture( GL::TEXTURE0 );
    gl.bind_texture( GL::TEXTURE_2D, fb.offscreen_color.as_ref() );
    shaders.screen.activate();
    gl.draw_arrays( GL::TRIANGLES, 0, 3 );

    true // Continue the render loop
  };
  // Run the update loop
  gl::exec_loop::run( update );

  Ok( () )
}

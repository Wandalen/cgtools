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

use minwebgl as gl;
use elliptical_orbit::EllipticalOrbit;
use renderer::webgl::{ loaders::gltf, AttributeInfo, IndexInfo };
use std::{ cell::RefCell, f32, rc::Rc };
use gl::
{
  GL,
  F32x3,
  BufferDescriptor,
  WebglError,
  geometry::BoundingBox,
  math::d2::mat3x3h,
  AsBytes as _,
  JsCast as _,
};
use web_sys::
{
  HtmlCanvasElement,
  WebGlFramebuffer,
  WebGlRenderbuffer,
  WebGlTexture
};

/// Parameters for the GUI controls
#[ derive( Debug, serde::Serialize, serde::Deserialize ) ]
pub struct GuiParams
{
  /// Number of active lights to render
  pub light_count : usize,
  /// RGB color of all lights
  pub light_color : [ f32; 3 ],
  /// Minimum radius for randomly generated light volumes
  pub min_radius : f32,
  /// Maximum radius for randomly generated light volumes
  pub max_radius : f32,
  /// Intensity multiplier for all lights
  pub intensity : f32,
}

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
  gl.viewport( 0, 0, width, height ); // Set the viewport
  gl.enable( GL::DEPTH_TEST ); // Enable depth testing
  gl.enable( GL::CULL_FACE ); // Enable face culling
  gl.cull_face( GL::BACK ); // Cull back faces
  gl.clear_color( 0.0, 0.0, 0.0, 1.0 ); // Set clear color
  gl.blend_func( gl::ONE, gl::ONE ); // Set blending function for additive blending

  // Load shaders
  // Shader for rendering light volumes (lighting pass)
  let vert = include_str!( "../shaders/light_volume.vert" );
  let frag = include_str!( "../shaders/deferred.frag" );
  let light_shader = gl::shader::Program::new( gl.clone(), vert, frag )?;

  // Shader for rendering scene objects (geometry pass)
  let vert = include_str!( "../shaders/main.vert" );
  let frag = include_str!( "../shaders/gbuffer.frag" );
  let object_shader = gl::shader::Program::new( gl.clone(), vert, frag )?;

  // Shader for drawing the final result to the screen
  let vert = include_str!( "../shaders/big_triangle.vert" );
  let frag = include_str!( "../shaders/screen_texture.frag" );
  let screen_shader = gl::shader::Program::new( gl.clone(), vert, frag )?;

  // Shader for visualizing light source positions
  let vert = include_str!( "../shaders/light_sphere.vert" );
  let frag = include_str!( "../shaders/light_sphere.frag" );
  let light_sphere_shader = gl::shader::Program::new( gl.clone(), vert, frag )?;

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

  // Create the G-buffer framebuffer and its textures (position, normal, color) and depth buffer
  let
  (
    gbuffer,
    position_gbuffer,
    normal_gbuffer,
    color_gbuffer,
    depthbuffer,
  ) = gbuffer( &gl, width, height );

  // Create an offscreen framebuffer for the final rendered image
  let offscreen_color = tex_storage_2d( &gl, GL::RGBA8, width, height );
  let offscreen_buffer = gl.create_framebuffer();
  gl.bind_framebuffer( GL::FRAMEBUFFER, offscreen_buffer.as_ref() );
  gl.framebuffer_texture_2d
  (
    GL::FRAMEBUFFER,
    GL::COLOR_ATTACHMENT0,
    GL::TEXTURE_2D,
    offscreen_color.as_ref(),
    0
  );

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

  // Light setup
  let max_light_count = 5000;

  // Generate random elliptical orbits for light sources movement
  let light_orbits = ( 0..max_light_count ).map
  (
    | _ |
    EllipticalOrbit
    {
      center : F32x3::new
      (
        rand::random_range( -30.0..=30.0 ),
        rand::random_range( -30.0..=30.0 ),
        rand::random_range( -110.0..=-90.0 )
      ),
      ..EllipticalOrbit::random()
    }
  ).collect::< Vec< _ > >();
  // Generate random offsets to make elliptical movement more diverse
  let offsets = ( 0..max_light_count )
  .map( | _ | rand::random_range( 0.0..=( f32::consts::PI * 2.0 ) ) ).collect::< Vec< _ > >();

  // Function to generate random radii for light volumes based on min/max range
  let generate_radii = move | min : f32, max : f32 | -> Vec< f32 >
  {
    let mut radii = ( 0..max_light_count )
    .map( | _ | rand::random_range( min..=max ) ).collect::< Vec< _ > >();
    // Set the first light as a large global light
    radii[ 0 ] = 100.0;
    radii
  };

  // Generate initial random radii for light volumes
  let light_radii = Rc::new( RefCell::new( generate_radii( params.min_radius, params.max_radius ) ) );
  // Initialize light translations (positions)
  let mut light_translations = vec![ [ 0.0f32, 0.0, 0.0 ]; max_light_count as usize ];
  // Set the first light as a large global light
  light_translations[ 0 ] = [ 0.0, 0.0, -100.0 ];

  // Track previous radius range to detect changes
  let prev_radius_range = Rc::new( RefCell::new( ( params.min_radius, params.max_radius ) ) );

  // Create the geometry for the light volume (a cube)
  let mut light_volume = light_volume( &gl )?;

  // Create and upload the buffer for light translations (used for instancing)
  let translation_buffer = gl::buffer::create( &gl )?;
  gl::buffer::upload( &gl, &translation_buffer, &light_translations, GL::DYNAMIC_DRAW );
  // Add the translation attribute to the light volume geometry
  let translation_attribute = AttributeInfo
  {
    slot : 1, // Attribute slot 1
    buffer : translation_buffer.clone(),
    descriptor : BufferDescriptor::new::< [ f32; 3 ] >().divisor( 1 ), // Instanced attribute
    bounding_box : BoundingBox::default(),
  };
  light_volume.add_attribute( &gl, "a_translation", translation_attribute, false )?;

  // Create and upload the buffer for light radii (used for instancing)
  let light_radius_buffer = gl::buffer::create( &gl )?;
  gl::buffer::upload( &gl, &light_radius_buffer, light_radii.borrow().as_slice(), GL::DYNAMIC_DRAW );
  // Add the radius attribute to the light volume geometry
  let radius_attribute = AttributeInfo
  {
    slot : 2, // Attribute slot 2
    buffer : light_radius_buffer.clone(),
    descriptor : BufferDescriptor::new::< f32 >().divisor( 1 ), // Instanced attribute
    bounding_box : BoundingBox::default(),
  };
  light_volume.add_attribute( &gl, "a_radius", radius_attribute, false )?;

  // Setup sphere geometry for light visualization
  // Extract the first mesh from the sphere GLTF
  let sphere_mesh = &sphere.meshes[ 0 ];
  let sphere_primitive = &sphere_mesh.borrow().primitives[ 0 ];
  let light_sphere_geometry = sphere_primitive.borrow().geometry.clone();

  // Add the translation attribute to the sphere geometry for instancing
  let sphere_translation_attribute = AttributeInfo
  {
    slot : 1, // Attribute slot 1
    buffer : translation_buffer.clone(),
    descriptor : BufferDescriptor::new::< [ f32; 3 ] >().divisor( 1 ), // Instanced attribute
    bounding_box : BoundingBox::default(),
  };
  light_sphere_geometry.borrow_mut().add_attribute( &gl, "a_translation", sphere_translation_attribute, false )?;

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
    let mut prev_range = prev_radius_range.borrow_mut();
    if prev_range.0 != params.min_radius || prev_range.1 != params.max_radius
    {
      // Regenerate light radii with new range (now guaranteed min <= max)
      let new_radii = generate_radii( params.min_radius, params.max_radius );
      *light_radii.borrow_mut() = new_radii;
      // Update the radius buffer
      gl.bind_buffer( GL::ARRAY_BUFFER, Some( &light_radius_buffer ) );
      gl.buffer_sub_data_with_i32_and_u8_array
      (
        GL::ARRAY_BUFFER,
        0,
        light_radii.borrow().as_bytes()
      );
      // Update prev range
      *prev_range = ( params.min_radius, params.max_radius );
    }
    drop( prev_range );
    // Update light positions based on their elliptical orbits
    // Start from index 1 because the first light is static (global)
    light_orbits[ 1..light_count ].iter().zip( offsets[ 1..light_count ].iter() ).enumerate()
    .for_each
    (
      | ( i, ( orbit, offset ) ) |
      light_translations[ i + 1 ] = orbit.position_at_angle( 0.3 * current_time + *offset ).0
    );
    // Update the translation buffer with the new light positions
    gl.bind_buffer( GL::ARRAY_BUFFER, Some( &translation_buffer ) );
    gl.buffer_sub_data_with_i32_and_u8_array_and_src_offset
    (
      GL::ARRAY_BUFFER,
      size_of::< [ f32; 3 ] >() as i32, // Offset to skip the first light source's data
      light_translations[ 1..light_count ].as_bytes(),
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
    gl.bind_framebuffer( GL::FRAMEBUFFER, gbuffer.as_ref() );
    // Attach the depth buffer to the G-buffer framebuffer
    gl.framebuffer_renderbuffer
    (
      GL::FRAMEBUFFER,
      GL::DEPTH_ATTACHMENT,
      GL::RENDERBUFFER,
      depthbuffer.as_ref()
    );
    // Specify which color attachments to draw to
    gl::drawbuffers::drawbuffers( &gl, &[ 0, 1, 2 ] );
    gl.clear( GL::DEPTH_BUFFER_BIT ); // Clear only the depth buffer

    // Activate the object shader and set uniforms
    object_shader.activate();
    object_shader.uniform_matrix_upload( "u_model", scene_transform.raw_slice(), true );
    object_shader.uniform_matrix_upload( "u_rotation", rotation.raw_slice(), true );
    object_shader.uniform_matrix_upload( "u_mvp", scene_mvp.raw_slice(), true );

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
    gl.bind_framebuffer( GL::FRAMEBUFFER, offscreen_buffer.as_ref() );
    // Attach the depth buffer from the G-buffer to the offscreen framebuffer
    gl.framebuffer_renderbuffer
    (
      GL::FRAMEBUFFER,
      GL::DEPTH_ATTACHMENT,
      GL::RENDERBUFFER,
      depthbuffer.as_ref()
    );
    // Specify which color attachment to draw to (only the first one for the final color)
    gl::drawbuffers::drawbuffers( &gl, &[ 0 ] );
    gl.clear( gl::COLOR_BUFFER_BIT ); // Clear the color buffer

    // Bind the G-buffer textures to texture units
    gl.active_texture( GL::TEXTURE0 );
    gl.bind_texture( GL::TEXTURE_2D, position_gbuffer.as_ref() );
    gl.active_texture( GL::TEXTURE1 );
    gl.bind_texture( GL::TEXTURE_2D, normal_gbuffer.as_ref() );
    gl.active_texture( GL::TEXTURE2 );
    gl.bind_texture( GL::TEXTURE_2D, color_gbuffer.as_ref() );

    // Bind the light volume geometry and activate the light shader
    light_volume.bind( &gl );
    light_shader.activate();
    // Set uniforms for the light shader
    light_shader.uniform_matrix_upload( "u_mvp", ( projection * view ).raw_slice(), true );
    light_shader.uniform_upload( "u_camera_position", camera.get_eye().as_slice() );
    light_shader.uniform_upload( "u_screen_size", [ width as f32, height as f32 ].as_slice() );
    // Set texture uniforms to the correct texture units
    light_shader.uniform_upload( "u_positions", &0 );
    light_shader.uniform_upload( "u_normals", &1 );
    light_shader.uniform_upload( "u_colors", &2 );
    // Set light color and intensity from GUI
    let light_color_with_intensity =
    [
      params.light_color[ 0 ] * params.intensity,
      params.light_color[ 1 ] * params.intensity,
      params.light_color[ 2 ] * params.intensity
    ];
    gl.vertex_attrib3f( 3, light_color_with_intensity[ 0 ], light_color_with_intensity[ 1 ], light_color_with_intensity[ 2 ] );

    // Draw the light volumes instanced
    light_volume.draw_instanced( &gl, light_count as i32 );

    // --- Light Source Visualization Pass ---
    // Draw small spheres at light positions to visualize light sources
    // This happens before the screen pass so spheres are properly occluded by geometry
    gl.disable( gl::BLEND ); // Disable blending for opaque sphere rendering
    gl.enable( GL::DEPTH_TEST );
    gl.depth_func( GL::LESS );
    gl.depth_mask( true );
    gl.cull_face( GL::BACK ); // Reset culling face

    // Keep offscreen_buffer bound and attach depth buffer for occlusion
    gl.bind_framebuffer( GL::FRAMEBUFFER, offscreen_buffer.as_ref() );
    gl.framebuffer_renderbuffer
    (
      GL::FRAMEBUFFER,
      GL::DEPTH_ATTACHMENT,
      GL::RENDERBUFFER,
      depthbuffer.as_ref()
    );

    // Bind the sphere geometry
    light_sphere_geometry.borrow().bind( &gl );
    // Activate the light sphere shader
    light_sphere_shader.activate();
    // Set uniforms
    light_sphere_shader.uniform_matrix_upload( "u_view_projection", ( projection * view ).raw_slice(), true );
    light_sphere_shader.uniform_upload( "u_scale", &0.2_f32 ); // Small sphere scale
    light_sphere_shader.uniform_upload( "u_color", params.light_color.as_slice() );

    // Draw the spheres instanced at each light position
    light_sphere_geometry.borrow().draw_instanced( &gl, light_count as i32 );

    // Disable depth test for the final screen pass
    gl.disable( GL::DEPTH_TEST );

    // --- Final Screen Pass ---
    // Bind the default framebuffer (the canvas)
    gl.bind_framebuffer( GL::FRAMEBUFFER, None );
    // Bind the offscreen color texture to texture unit 0
    gl.active_texture( GL::TEXTURE0 );
    gl.bind_texture( GL::TEXTURE_2D, offscreen_color.as_ref() );
    // Activate the screen shader
    screen_shader.activate();
    // Draw a big triangle that covers the entire screen to display the offscreen texture
    gl.draw_arrays( GL::TRIANGLES, 0, 3 );

    true // Continue the render loop
  };
  // Run the update loop
  gl::exec_loop::run( update );

  Ok( () )
}

/// Creates and configures the G-buffer framebuffer and its associated textures
/// (position, normal, color) and depth renderbuffer.
fn gbuffer( gl : &GL, width : i32, height : i32 )
->
(
  Option< WebGlFramebuffer >,
  Option< WebGlTexture >,
  Option< WebGlTexture >,
  Option< WebGlTexture >,
  Option< WebGlRenderbuffer >
)
{
  // Create textures for position, normal, and color
  // RGBA16F for position and normal to store floating-point data
  let position_gbuffer = tex_storage_2d( gl, GL::RGBA16F, width, height );
  let normal_gbuffer = tex_storage_2d( gl, GL::RGBA16F, width, height );
  // RGBA8 for color (standard 8-bit per channel)
  let color_gbuffer = tex_storage_2d( gl, GL::RGBA8, width, height );

  // Create a renderbuffer for depth
  let depthbuffer = gl.create_renderbuffer();
  gl.bind_renderbuffer( GL::RENDERBUFFER, depthbuffer.as_ref() );
  gl.renderbuffer_storage( GL::RENDERBUFFER, GL::DEPTH_COMPONENT24, width, height );

  // Create the framebuffer
  let gbuffer = gl.create_framebuffer();
  gl.bind_framebuffer( GL::FRAMEBUFFER, gbuffer.as_ref() );

  // Attach the textures and depth buffer to the framebuffer's attachment points
  gl.framebuffer_texture_2d( GL::FRAMEBUFFER, GL::COLOR_ATTACHMENT0, GL::TEXTURE_2D, position_gbuffer.as_ref(), 0 );
  gl.framebuffer_texture_2d( GL::FRAMEBUFFER, GL::COLOR_ATTACHMENT1, GL::TEXTURE_2D, normal_gbuffer.as_ref(), 0 );
  gl.framebuffer_texture_2d( GL::FRAMEBUFFER, GL::COLOR_ATTACHMENT2, GL::TEXTURE_2D, color_gbuffer.as_ref(), 0 );
  gl.framebuffer_renderbuffer( GL::FRAMEBUFFER, GL::DEPTH_ATTACHMENT, GL::RENDERBUFFER, depthbuffer.as_ref() );

  // Return the created framebuffer and attachments
  ( gbuffer, position_gbuffer, normal_gbuffer, color_gbuffer, depthbuffer )
}

/// Helper function to create a 2D texture with specified format, width, and height,
/// and set its filtering to nearest.
fn tex_storage_2d( gl : &GL, format : u32, width : i32, height : i32 ) -> Option< WebGlTexture >
{
  let tex = gl.create_texture()?;
  gl.bind_texture( GL::TEXTURE_2D, Some( &tex ) );
  // Allocate texture storage
  gl.tex_storage_2d( GL::TEXTURE_2D, 1, format, width, height );
  // Set texture filtering to nearest (important for G-buffer sampling)
  gl::texture::d2::filter_nearest( gl );
  Some( tex )
}

/// Creates the geometry for a light volume (a unit cube).
fn light_volume( gl : &GL ) -> Result< renderer::webgl::Geometry, WebglError >
{
  // Define cube vertices
  static CUBE_VERTICES : &[ f32 ] =
  &[
    // Front face
    -1.0, -1.0,  1.0,
     1.0, -1.0,  1.0,
     1.0,  1.0,  1.0,
    -1.0,  1.0,  1.0,
    // Back face
    -1.0, -1.0, -1.0,
     1.0, -1.0, -1.0,
     1.0,  1.0, -1.0,
    -1.0,  1.0, -1.0,
  ];

  // Define cube indices
  static CUBE_INDICES : &[ u32 ] =
  &[
    // Front face
    0, 1, 2, 0, 2, 3,
    // Back face
    4, 6, 5, 4, 7, 6,
    // Top face
    3, 2, 6, 3, 6, 7,
    // Bottom face
    0, 5, 1, 0, 4, 5,
    // Right face
    1, 5, 6, 1, 6, 2,
    // Left face
    0, 3, 7, 0, 7, 4,
  ];

  // Unbind any currently bound vertex array
  gl.bind_vertex_array( None );

  // Create a new Geometry object
  let mut light_volume = renderer::webgl::Geometry::new( &gl )?;

  // Create and upload the position buffer
  let position_buffer = gl::buffer::create( gl )?;
  gl::buffer::upload( gl, &position_buffer, CUBE_VERTICES, GL::STATIC_DRAW );
  // Add the position attribute to the geometry
  let attribute = AttributeInfo
  {
    slot : 0, // Attribute slot 0
    buffer : position_buffer,
    descriptor : BufferDescriptor::new::< [ f32; 3 ] >(), // Non-instanced attribute
    bounding_box : BoundingBox::default(),
  };
  light_volume.add_attribute( &gl, "position", attribute, false )?;

  // Create and upload the index buffer
  let index_buffer = gl::buffer::create( gl )?;
  gl::index::upload( gl, &index_buffer, CUBE_INDICES, GL::STATIC_DRAW );
  // Add the index buffer to the geometry
  let index = IndexInfo
  {
    buffer : index_buffer,
    count : CUBE_INDICES.len() as u32,
    offset : 0,
    data_type : GL::UNSIGNED_INT,
  };
  light_volume.add_index( &gl, index )?;

  Ok( light_volume )
}

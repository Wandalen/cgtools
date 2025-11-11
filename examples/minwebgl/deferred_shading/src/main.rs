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

use elliptical_orbit::EllipticalOrbit;
use renderer::webgl::{ AttributeInfo, IndexInfo, loaders::gltf, material::PBRMaterial };
use std::{ cell::RefCell, f32, rc::Rc };
use minwebgl as gl;
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
  wasm_bindgen::prelude::Closure,
  Event,
  HtmlCanvasElement,
  HtmlInputElement,
  WebGlFramebuffer,
  WebGlRenderbuffer,
  WebGlTexture
};

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

  // Setup camera and scene transformations
  let projection = mat3x3h::perspective_rh_gl( 60.0f32.to_radians(), aspect, 0.1, 1000.0 );
  let rotation = mat3x3h::rot( 10.0f32.to_radians(), 0.0, 0.0 )
  * mat3x3h::rot( 0.0, 90.0f32.to_radians(), 0.0 );
  let scale = 0.1;
  let scene_transform = mat3x3h::translation( [ 0.0f32, -40.0, -95.0 ] )
  * rotation
  * mat3x3h::scale( [ scale, scale, scale ] );

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

  // Light setup
  let max_light_count = 5000;
  // Use Rc<RefCell> to share and mutate the light count from the slider
  let light_count = Rc::new( RefCell::new( 200 ) );
  let light_radius = 12.0f32;

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
  // Generate random radii for light volumes
  let mut light_radii = ( 0..max_light_count )
  .map( | _ | light_radius + rand::random_range( -1.0..=7.0 ) ).collect::< Vec< _ > >();
  // Initialize light translations (positions)
  let mut light_translations = vec![ [ 0.0f32, 0.0, 0.0 ]; max_light_count as usize ];
  // Set the first light as a large global light
  light_radii[ 0 ] = 100.0;
  light_translations[ 0 ] = [ 0.0, 0.0, -100.0 ];

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
  light_volume.add_attribute( &gl, "a_translation", translation_attribute )?;

  // Create and upload the buffer for light radii (used for instancing)
  let light_radius_buffer = gl::buffer::create( &gl )?;
  gl::buffer::upload( &gl, &light_radius_buffer, light_radii.as_slice(), GL::STATIC_DRAW );
  // Add the radius attribute to the light volume geometry
  let radius_attribute = AttributeInfo
  {
    slot : 2, // Attribute slot 2
    buffer : light_radius_buffer,
    descriptor : BufferDescriptor::new::< f32 >().divisor( 1 ), // Instanced attribute
    bounding_box : BoundingBox::default(),
  };
  light_volume.add_attribute( &gl, "a_radius", radius_attribute )?;

  // Get UI elements
  let fps_counter = document.get_element_by_id( "fps-counter" ).unwrap();
  let slider_value = document.get_element_by_id( "slider-value" ).unwrap();
  let slider = document.get_element_by_id( "slider" ).unwrap().dyn_into::< HtmlInputElement >().unwrap();

  // Create a closure for the slider's onchange event
  let onchange = Closure::< dyn Fn( _ ) >::new
  (
    {
      let light_count = light_count.clone();
      let slider_value = slider_value.clone();
      move | e : Event |
      {
        // Get the new light count from the slider value
        let num = e.target().unwrap()
        .dyn_into::< HtmlInputElement >().unwrap().value_as_number() as usize;
        // Update the shared light count
        *light_count.borrow_mut() = num;
        // Update the displayed slider value
        slider_value.set_text_content( Some( &num.to_string() ) );
      }
    }
  );
  // Attach the closure to the slider's onchange event
  slider.set_onchange( Some( onchange.as_ref().unchecked_ref() ) );
  // Forget the closure to prevent it from being dropped
  onchange.forget();

  // Variables for FPS calculation and camera animation
  let mut last_time = 0.0;
  let mut fps = 0;
  let mut camera_direction = 1.0f32;
  let mut camera_y = 0.0f32;

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
    let delta_time = current_time - last_time;
    last_time = current_time;
    fps += 1;

    let light_count = *light_count.borrow();
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

    // Simple camera vertical movement animation
    if camera_y < -20.0 { camera_direction = 1.0; }
    if camera_y > 60.0 { camera_direction = -1.0; }
    camera_y += delta_time * camera_direction * 10.0;
    let view = mat3x3h::translation( [ 0.0, -camera_y, 0.0 ]);
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
        let material = renderer::webgl::helpers::cast_unchecked_material_to_ref::< PBRMaterial >( material );
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
    light_shader.uniform_upload( "u_camera_position", [ 0.0, camera_y, 0.0 ].as_slice() );
    light_shader.uniform_upload( "u_screen_size", [ width as f32, height as f32 ].as_slice() );
    // Set texture uniforms to the correct texture units
    light_shader.uniform_upload( "u_positions", &0 );
    light_shader.uniform_upload( "u_normals", &1 );
    light_shader.uniform_upload( "u_colors", &2 );
    // Set a constant light color (can be made dynamic)
    gl.vertex_attrib3f( 3, 0.5, 0.5, 0.5 ); // Assuming attribute 3 is for light color

    // Draw the light volumes instanced
    light_volume.draw_instanced( &gl, light_count as i32 );

    // Disable blending and depth test for the final screen pass
    gl.disable( gl::BLEND );
    gl.disable( GL::DEPTH_TEST );
    gl.cull_face( GL::BACK ); // Reset culling face

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
  light_volume.add_attribute( &gl, "position", attribute )?;

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

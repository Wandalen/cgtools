//! Implementation of JFA outline using WebGL2 and web_sys.
//!
//! This example demonstrates how to render a 3D object and generate a real-time
//! outline around it using the Jump Flooding Algorithm ( JFA ).
//!
//! The process involves several rendering passes:

#![ allow( clippy::needless_borrow ) ]
#![ allow( clippy::single_match ) ]
#![ allow( clippy::doc_overindented_list_items ) ]
//! 1. **Object Pass:** Render the 3D object to a texture ( framebuffer ) to get a silhouette.
//!    Object pixels are marked ( e.g., white ), background is clear.
//! 2. **JFA Initialization Pass:** Initialize a JFA texture. Pixels corresponding
//!    to the object silhouette store their own texture coordinates ( these are the "seeds" ).
//!    Background pixels store a sentinel value ( e.g., ( -1.0, -1.0 ) ).
//! 3. **JFA Step Passes:** Repeatedly apply the JFA step shader. In each pass,
//!    each pixel samples its neighbors at an decreasing jump distance. It updates
//!    its stored coordinate to the one belonging to the *nearest* "seed" found so far.
//!    This propagates the nearest seed coordinate outwards from the object silhouette.
//!    A ping-pong rendering strategy is used between two framebuffers.
//! 4. **Outline Pass:** Render a final screen-filling quad. Sample the original object
//!    silhouette texture and the final JFA texture. For background pixels, calculate
//!    the distance to the nearest seed ( using the coordinate stored in the JFA texture ).
//!    If this distance is within a defined thickness, draw the outline color; otherwise,
//!    draw the background color. Object pixels are drawn with the object color.

use minwebgl as gl;
use gl::
{
  WebglError,
  GL,
  JsCast,
  wasm_bindgen::closure::Closure,
  web_sys::
  {
    WebGl2RenderingContext,
    WebGlUniformLocation,
    WebGlTexture,
    WebGlFramebuffer
  }
};
use std::rc::Rc;
use std::cell::RefCell;
use renderer::webgl::
{
  loaders::gltf::load,
  scene::Scene,
  camera::Camera,
  Node,
  Object3D,
  program::
  {
    ProgramInfo,
    ShaderProgram,
    JfaOutlineObjectShader,
    JfaOutlineInitShader,
    JfaOutlineStepShader,
    JfaOutlineShader
  }
};
use ndarray_cg::F32x3;
use std::collections::HashMap;

/// Removes node from [`Scene`] by name
fn remove_node_from_scene_by_name( root : &Rc< RefCell< Scene > >, name : &str )
{
  let remove_child_ids = root.borrow().children
  .iter()
  .enumerate()
  .filter
  (
    | ( _, n ) |
    {
      if let Some( current_name ) = n.borrow().get_name()
      {
        *current_name == *name
      }
      else
      {
        false
      }
    }
  )
  .map( | ( i, _ ) | i )
  .collect::< Vec< _ > >();

  for i in remove_child_ids.iter().rev()
  {
    let _ = root.borrow_mut().children.remove( *i );
  }

  let _ = root.borrow_mut().traverse
  (
    &mut | node : Rc< RefCell< Node > > |
    {
      let remove_child_ids = node.borrow().get_children()
      .iter()
      .enumerate()
      .filter
      (
        | ( _, n ) |
        {
          if let Some( current_name ) = n.borrow().get_name()
          {
            *current_name == *name
          }
          else
          {
            false
          }
        }
      )
      .map( | ( i, _ ) | i )
      .collect::< Vec< _ > >();

      for i in remove_child_ids.iter().rev()
      {
        let _ = node.borrow_mut().remove_child( *i );
      }

      Ok( () )
    }
  );
}

/// Loads an image from a URL to a WebGL texture.
///
/// This function creates a new `WebGlTexture` and asynchronously loads an image from the provided URL into it.
/// It uses a `Closure` to handle the `onload` event of an `HtmlImageElement`, ensuring the texture is
/// uploaded only after the image has finished loading.
///
/// # Arguments
///
/// * `gl` - The WebGl2RenderingContext.
/// * `src` - A reference-counted string containing the URL of the image to load.
///
/// # Returns
///
/// A `WebGlTexture` object.
fn load_texture( gl : &GL, src : &str ) -> WebGlTexture
{
  let window = web_sys::window().expect( "Can't get window" );
  let document =  window.document().expect( "Can't get document" );

  let texture = gl.create_texture().expect( "Failed to create a texture" );

  let img_element = document.create_element( "img" )
  .expect( "Can't create img" )
  .dyn_into::< gl::web_sys::HtmlImageElement >()
  .expect( "Can't convert to gl::web_sys::HtmlImageElement" );
  img_element.style().set_property( "display", "none" ).expect( "Can't set property" );
  let load_texture : Closure< dyn Fn() > = Closure::new
  (
    {
      let gl = gl.clone();
      let img = img_element.clone();
      let texture = texture.clone();
      move ||
      {
        gl::texture::d2::upload_no_flip( &gl, Some( &texture ), &img );
        gl.generate_mipmap( gl::TEXTURE_2D );
        img.remove();
      }
    }
  );

  img_element.set_onload( Some( load_texture.as_ref().unchecked_ref() ) );
  img_element.set_src( &src );
  load_texture.forget();

  texture
}

/// Binds a texture to a texture unit and uploads its location to a uniform.
///
/// # Arguments
///
/// * `gl` - The WebGL2 rendering context.
/// * `texture` - The texture to bind.
/// * `location` - The uniform location in the shader for the sampler.
/// * `slot` - The texture unit to bind to ( e.g., `GL::TEXTURE0` ).
fn upload_texture
(
  gl : &gl::WebGl2RenderingContext,
  texture : &WebGlTexture,
  location : &WebGlUniformLocation,
  slot : u32,
)
{
  gl.active_texture( slot );
  gl.bind_texture( GL::TEXTURE_2D, Some( &texture ) );
  // Tell the sampler uniform in the shader which texture unit to use ( 0 for GL_TEXTURE0, 1 for GL_TEXTURE1, etc. )
  gl.uniform1i( Some( location ), ( slot - GL::TEXTURE0 ) as i32 );
}

/// Creates a WebGL2 framebuffer and a color attachment texture.
///
/// # Arguments
///
/// * `gl` - The WebGL2 rendering context.
/// * `size` - The size of the framebuffer and its attachment ( width, height ).
/// * `color_attachment` - The index of the color attachment point ( e.g., 0 for `GL::COLOR_ATTACHMENT0` ).
///
/// # Returns
///
/// An `Option< ( WebGlFramebuffer, WebGlTexture ) >` containing the created framebuffer and
/// its color attachment texture, or `None` if creation fails.
fn create_framebuffer
(
  gl : &gl::WebGl2RenderingContext,
  size : ( i32, i32 ),
  enable_normal_target : bool
)
-> Option< ( WebGlFramebuffer, WebGlTexture, Option< WebGlTexture > ) >
{
  let color = gl.create_texture()?;

  gl.bind_texture( GL::TEXTURE_2D, Some( &color ) );
  // Use tex_storage_2d for immutable texture storage ( WebGL2 )
  gl.tex_storage_2d( GL::TEXTURE_2D, 1, gl::RGBA8, size.0, size.1 );
  // Configure texture parameters (filtering, wrapping)
  gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_MIN_FILTER, GL::NEAREST as i32 );
  gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_WRAP_S, GL::REPEAT as i32 );
  gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_WRAP_T, GL::REPEAT as i32 );

  let ( normal, depthbuffer ) = if enable_normal_target
  {
    let depthbuffer = gl.create_renderbuffer();
    gl.bind_renderbuffer( GL::RENDERBUFFER, depthbuffer.as_ref() );
    gl.renderbuffer_storage( GL::RENDERBUFFER, GL::DEPTH_COMPONENT24, size.0, size.1 );

    let normal = gl.create_texture()?;
    gl.bind_texture( GL::TEXTURE_2D, Some( &normal ) );
    // Use tex_storage_2d for immutable texture storage ( WebGL2 )
    gl.tex_storage_2d( GL::TEXTURE_2D, 1, gl::RGBA16F, size.0, size.1 );
    // Configure texture parameters (filtering, wrapping)
    gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_MIN_FILTER, GL::LINEAR as i32 );
    gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_WRAP_S, GL::REPEAT as i32 );
    gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_WRAP_T, GL::REPEAT as i32 );
    ( Some( normal ), depthbuffer )
  }
  else
  {
    ( None, None )
  };

  let framebuffer = gl.create_framebuffer()?;
  gl.bind_framebuffer( GL::FRAMEBUFFER, Some( &framebuffer ) );
  // Attach the texture to the framebuffer's color attachment point
  gl.framebuffer_texture_2d( GL::FRAMEBUFFER, GL::COLOR_ATTACHMENT0, GL::TEXTURE_2D, Some( &color ), 0 );
  if enable_normal_target
  {
    gl.framebuffer_texture_2d( GL::FRAMEBUFFER, GL::COLOR_ATTACHMENT1, GL::TEXTURE_2D, normal.as_ref(), 0 );
    gl.framebuffer_renderbuffer( GL::FRAMEBUFFER, GL::DEPTH_ATTACHMENT, GL::RENDERBUFFER, depthbuffer.as_ref() );
  }
  gl::drawbuffers::drawbuffers( gl, &[ 0, 1 ] );
  gl.viewport( 0, 0, size.0, size.1 );
  gl.bind_framebuffer( gl::FRAMEBUFFER, None );

  Some( ( framebuffer, color, normal ) )
}

/// Binds a framebuffer for rendering and sets the viewport to its size.
///
/// # Arguments
///
/// * `gl` - The WebGL2 rendering context.
/// * `framebuffer` - The framebuffer to bind.
/// * `size` - The size of the framebuffer ( width, height ).
fn upload_framebuffer(
  gl : &gl::WebGl2RenderingContext,
  framebuffer : &WebGlFramebuffer,
  size : ( i32, i32 )
)
{
  gl.bind_framebuffer( GL::FRAMEBUFFER, Some( framebuffer ) );
  gl.viewport( 0, 0, size.0, size.1 );
}

struct Programs
{
  object : ProgramInfo,
  jfa_init : ProgramInfo,
  jfa_step : ProgramInfo,
  outline : ProgramInfo
}

impl Programs
{
  fn new( gl : &gl::WebGl2RenderingContext ) -> Self
  {
    // --- Load and Compile Shaders ---

    let object_vs_src = include_str!( "../resources/shaders/object.vert" );
    let object_fs_src = include_str!( "../resources/shaders/object.frag" );
    let fullscreen_vs_src = include_str!( "../resources/shaders/fullscreen.vert" );
    let jfa_init_fs_src = include_str!( "../resources/shaders/jfa_init.frag" );
    let jfa_step_fs_src = include_str!( "../resources/shaders/jfa_step.frag" );
    let outline_fs_src = include_str!( "../resources/shaders/outline.frag" );

    // Compile and link shader programs and store them
    let object_program = gl::ProgramFromSources::new( object_vs_src, object_fs_src ).compile_and_link( gl ).unwrap();
    let jfa_init_program = gl::ProgramFromSources::new( fullscreen_vs_src, jfa_init_fs_src ).compile_and_link( gl ).unwrap();
    let jfa_step_program = gl::ProgramFromSources::new( fullscreen_vs_src, jfa_step_fs_src ).compile_and_link( gl ).unwrap();
    let outline_program = gl::ProgramFromSources::new( fullscreen_vs_src, outline_fs_src ).compile_and_link( gl ).unwrap();

    let object = ProgramInfo::new( gl, &object_program, JfaOutlineObjectShader.dyn_clone() );
    let jfa_init = ProgramInfo::new( gl, &jfa_init_program, JfaOutlineInitShader.dyn_clone() );
    let jfa_step = ProgramInfo::new( gl, &jfa_step_program, JfaOutlineStepShader.dyn_clone() );
    let outline = ProgramInfo::new( gl, &outline_program, JfaOutlineShader.dyn_clone() );

    Self
    {
      object,
      jfa_init,
      jfa_step,
      outline
    }
  }
}

/// Manages WebGL resources and rendering passes.
struct Renderer
{
  gl : WebGl2RenderingContext,
  programs : Programs,
  textures : HashMap< String, WebGlTexture >,
  framebuffers : HashMap< String, WebGlFramebuffer >,
  viewport : ( i32, i32 ),
  camera : Camera
}

impl Renderer
{
  /// Creates a new Renderer instance, initializes WebGL, loads resources,
  /// and prepares the scene for rendering.
  async fn new() -> Self
  {
    gl::browser::setup( Default::default() );
    let canvas = gl::canvas::make().unwrap();
    let gl = gl::context::from_canvas( &canvas ).unwrap();

    let _ = gl.get_extension( "EXT_color_buffer_float" ).unwrap();

    // --- Initialization ---

    let viewport = ( gl.drawing_buffer_width(), gl.drawing_buffer_height() );

    let eye = F32x3::from_array( [ 0.0, 1.0, 1.0 ] );
    let up = F32x3::Y;

    let aspect_ratio = viewport.0 as f32 / viewport.1 as f32;
    let fov =  70.0f32.to_radians();
    let near = 0.0001;
    let far = 1000.0;

    let camera = Camera::new(
      eye,
      up,
      F32x3::new( 0.0, 0.0, 0.0 ),
      aspect_ratio,
      fov,
      near,
      far
    );

    camera.bind_controls( &canvas );

    let programs = Programs::new( &gl );

    // Create and store renderer instance
    let mut renderer = Self
    {
      gl,
      programs,
      textures : HashMap::new(),
      framebuffers : HashMap::new(),
      viewport,
      camera
    };

    let gl = &renderer.gl;

    // --- Create Framebuffers and Textures ---

    // Framebuffer for rendering the initial object silhouette
    let ( object_fb, object_fb_color, object_fb_normal ) = create_framebuffer( gl, viewport, true ).unwrap();
    // Framebuffer for the JFA initialization pass
    let ( jfa_init_fb, jfa_init_fb_color, _ ) = create_framebuffer( gl, viewport, false ).unwrap();
    // Framebuffers for the JFA step passes ( ping-pong )
    let ( jfa_step_fb_0, jfa_step_fb_color_0, _ ) = create_framebuffer( gl, viewport, false ).unwrap();
    let ( jfa_step_fb_1, jfa_step_fb_color_1, _ ) = create_framebuffer( gl, viewport, false ).unwrap();

    // Store the color attachment textures
    renderer.textures.insert( "object_fb_color".to_string(), object_fb_color );
    renderer.textures.insert( "object_fb_normal".to_string(), object_fb_normal.unwrap() );
    renderer.textures.insert( "jfa_init_fb_color".to_string(), jfa_init_fb_color );
    renderer.textures.insert( "jfa_step_fb_color_0".to_string(), jfa_step_fb_color_0 );
    renderer.textures.insert( "jfa_step_fb_color_1".to_string(), jfa_step_fb_color_1 );
    renderer.textures.insert( "equirect_map".to_string(), load_texture( gl, "static/skybox/pink_sunrise.jpg" ) );

    // Store the framebuffers
    renderer.framebuffers.insert( "object_fb".to_string(), object_fb );
    renderer.framebuffers.insert( "jfa_init_fb".to_string(), jfa_init_fb );
    renderer.framebuffers.insert( "jfa_step_fb_0".to_string(), jfa_step_fb_0 );
    renderer.framebuffers.insert( "jfa_step_fb_1".to_string(), jfa_step_fb_1 );

    renderer
  }

  /// Executes all rendering passes for a single frame.
  ///
  /// # Arguments
  ///
  /// * `t` - The current time in milliseconds ( used for animation ).
  fn render( &self, scene : Rc< RefCell< Scene > >, t : f64 )
  {
    // 2. Object Rendering Pass: Render the object silhouette to a texture
    let _ = self.object_pass( scene );
    // 3. JFA Initialization Pass: Initialize JFA texture from the silhouette
    self.jfa_init_pass();

    // 4. JFA Step Passes: Perform Jump Flooding Algorithm steps
    // The number of passes required is log2( max( width, height ) ).
    let num_passes = 4;
    for i in 0..num_passes
    {
      self.jfa_step_pass( i, t );
    }

    // 4. Outline Pass: Generate and render the final scene with the outline
    self.outline_pass( num_passes );
  }

  /// Renders the 3D object silhouette to the `object_fb`.
  ///
  /// Sets up the model-view-projection matrices and draws the loaded mesh.
  /// The fragment shader for this pass simply outputs white.
  ///
  /// # Arguments
  ///
  /// * `t` - The current time in milliseconds ( used for rotating the camera/view ).
  fn object_pass( &self, scene : Rc< RefCell< Scene > > ) -> Result< (), WebglError >
  {
    let gl = &self.gl;

    let object_fb = self.framebuffers.get( "object_fb" ).unwrap();

    let locations = self.programs.object.get_locations();

    let u_projection_loc = locations.get( "u_projection" ).unwrap().clone().unwrap();
    let u_view_loc = locations.get( "u_view" ).unwrap().clone().unwrap();
    let u_model_loc = locations.get( "u_model" ).unwrap().clone().unwrap();

    upload_framebuffer( gl, object_fb, self.viewport );

    gl::drawbuffers::drawbuffers( gl, &[ 0, 1 ] );
    gl.clear_bufferfv_with_f32_array( gl::COLOR, 0, &[ 0.0, 0.0, 0.0, 0.0 ] );
    gl.clear_bufferfv_with_f32_array( gl::COLOR, 1, &[ 0.0, 0.0, 0.0, 0.0 ] );
    gl.enable( GL::DEPTH_TEST );
    gl.clear( GL::DEPTH_BUFFER_BIT );
    gl.depth_mask( true );
    gl.depth_func( GL::LEQUAL );
    // gl.front_face( GL::CW );

    self.programs.object.bind( gl );

    // Define a closure to handle the drawing of each node in the scene.
    let mut draw_node =
    |
      node : Rc< RefCell< Node > >
    | -> Result< (), gl::WebglError >
    {
      node.borrow().upload( gl, self.programs.object.get_locations() );
      // If the node contains a mesh...
      if let Object3D::Mesh( ref mesh ) = node.borrow().object
      {
        // Iterate over each primitive in the mesh.
        for primitive_rc in mesh.borrow().primitives.iter()
        {
          let primitive = primitive_rc.borrow();

          gl::uniform::matrix_upload( gl, Some( u_projection_loc.clone() ), &self.camera.get_projection_matrix().to_array(), true ).unwrap();
          gl::uniform::matrix_upload( gl, Some( u_view_loc.clone() ), &self.camera.get_view_matrix().to_array(), true ).unwrap();
          gl::uniform::matrix_upload( gl, Some( u_model_loc.clone() ), &node.borrow().get_world_matrix().to_array(), true ).unwrap();

          primitive.bind( gl );
          primitive.draw( gl );
        }
      }

      Ok( () )
    };

    // Traverse the scene and draw all opaque objects.
    let _ = scene.borrow().traverse( &mut draw_node )?;

    gl.use_program( None );
    gl.bind_framebuffer( GL::FRAMEBUFFER, None );

    Ok( () )
  }

  /// Performs the JFA initialization pass.
  ///
  /// Reads the object silhouette texture and writes texture coordinates for
  /// object pixels and a sentinel value for background pixels to the
  /// `jfa_init_fb`.
  fn jfa_init_pass( &self )
  {
    let gl = &self.gl;

    let jfa_init_fb = self.framebuffers.get( "jfa_init_fb" ).unwrap();
    let object_fb_color = self.textures.get( "object_fb_color" ).unwrap();

    self.programs.jfa_init.bind( gl );
    let locations = self.programs.jfa_init.get_locations();

    let u_object_texture = locations.get( "u_object_texture" ).unwrap().clone().unwrap();

    upload_framebuffer( gl, jfa_init_fb, self.viewport );

    upload_texture( gl, object_fb_color, &u_object_texture, GL::TEXTURE0 );

    gl.draw_arrays( GL::TRIANGLES, 0, 3 );
  }

  /// Performs one step of the Jump Flooding Algorithm.
  ///
  /// Reads from the JFA texture of the previous step and writes to one of the
  /// ping-pong JFA framebuffers ( `jfa_step_fb_0` or `jfa_step_fb_1` ).
  ///
  /// # Arguments
  ///
  /// * `i` - The current JFA step index ( 0, 1, 2, ... ).
  /// * `last` - A boolean flag. If true, the result of this step is rendered
  ///            directly to the default framebuffer ( screen ) for debugging.
  fn jfa_step_pass( &self, i : i32, t : f64 )
  {
    let gl = &self.gl;

    let jfa_step_fb_0 = self.framebuffers.get( "jfa_step_fb_0" ).unwrap();
    let jfa_step_fb_1 = self.framebuffers.get( "jfa_step_fb_1" ).unwrap();
    let jfa_init_fb_color = self.textures.get( "jfa_init_fb_color" ).unwrap(); // Initial JFA texture
    let jfa_step_fb_color_0 = self.textures.get( "jfa_step_fb_color_0" ).unwrap(); // Color texture for FB 0
    let jfa_step_fb_color_1 = self.textures.get( "jfa_step_fb_color_1" ).unwrap(); // Color texture for FB 1

    self.programs.jfa_step.bind( gl );
    let locations = self.programs.jfa_step.get_locations();

    let u_resolution = locations.get( "u_resolution" ).unwrap().clone().unwrap();
    let u_step_size = locations.get( "u_step_size" ).unwrap().clone().unwrap();
    let u_jfa_init_texture = locations.get( "u_jfa_texture" ).unwrap().clone().unwrap();

    // Ping-pong rendering: Determine input texture and output framebuffer based on step index `i`
    if i == 0 // First step uses the initialization result
    {
      upload_framebuffer( gl, jfa_step_fb_0, self.viewport ); // Render to FB 0
      upload_texture( gl, jfa_init_fb_color, &u_jfa_init_texture, GL::TEXTURE0 ); // Input is JFA init texture
    }
    else if i % 2 == 0 // Even steps ( 2, 4, ... ) read from FB 1, render to FB 0
    {
      upload_framebuffer( gl, jfa_step_fb_0, self.viewport ); // Render to FB 0
      upload_texture( gl, &jfa_step_fb_color_1, &u_jfa_init_texture, GL::TEXTURE0 ); // Input is texture from FB 1
    }
    else // Odd steps ( 1, 3, ... ) read from FB 0, render to FB 1
    {
      upload_framebuffer( gl, jfa_step_fb_1, self.viewport ); // Render to FB 1
      upload_texture( gl, jfa_step_fb_color_0, &u_jfa_init_texture, GL::TEXTURE0 ); // Input is texture from FB 0
    }

    // Upload resolution uniform ( needed for distance calculations in the shader )
    gl::uniform::upload( gl, Some( u_resolution.clone() ), &[ self.viewport.0 as f32, self.viewport.1 as f32 ] ).unwrap();

    let aspect_ratio = self.viewport.0 as f32 / self.viewport.1 as f32;
    let step_size = ( 5.0 * ( t as f32 / 500.0 ).sin().abs() ) / ( 2.0_f32 ).powf( i as f32 );
    let step_size = ( step_size * aspect_ratio, step_size );

    gl::uniform::upload( gl, Some( u_step_size.clone() ), &[ step_size.0, step_size.1 ] ).unwrap();

    gl.draw_arrays( GL::TRIANGLES, 0, 3 );
  }

  /// Performs the final outline pass.
  ///
  /// Reads the original object silhouette texture and the final JFA result texture
  /// to draw the final scene with object color, outline color, or background color.
  /// Renders to the default framebuffer ( screen ).
  ///
  /// # Arguments
  ///
  /// * `t` - The current time in milliseconds ( used for animating outline thickness ).
  /// * `num_passes` - The total number of JFA step passes performed. Used to determine
  ///                which of the ping-pong textures ( `jfa_step_fb_color_0` or `jfa_step_fb_color_1` )
  ///                holds the final JFA result.
  fn outline_pass( &self, num_passes : i32 )
  {
    let gl = &self.gl;

    let object_fb_color = self.textures.get( "object_fb_color" ).unwrap(); // Original silhouette
    let object_fb_normal = self.textures.get( "object_fb_normal" ).unwrap(); // Normal buffer
    let jfa_step_fb_color_0 = self.textures.get( "jfa_step_fb_color_0" ).unwrap(); // JFA ping-pong texture 0
    let jfa_step_fb_color_1 = self.textures.get( "jfa_step_fb_color_1" ).unwrap(); // JFA ping-pong texture 1
    let equirect_map = self.textures.get( "equirect_map" ).unwrap(); // Skybox equirectangular texture

    self.programs.outline.bind( gl );
    let locations = self.programs.outline.get_locations();

    let u_object_texture = locations.get( "u_object_texture" ).unwrap().clone().unwrap();
    let u_normal_texture = locations.get( "u_normal_texture" ).unwrap().clone().unwrap();
    let u_jfa_step_texture = locations.get( "u_jfa_texture" ).unwrap().clone().unwrap();
    let u_resolution = locations.get( "u_resolution" ).unwrap().clone().unwrap();
    let u_outline_thickness = locations.get( "u_outline_thickness" ).unwrap().clone().unwrap();
    let u_outline_color = locations.get( "u_outline_color" ).unwrap().clone().unwrap();
    let u_object_color = locations.get( "u_object_color" ).unwrap().clone().unwrap();
    let u_equirect_map = locations.get( "u_equirect_map" ).unwrap().clone().unwrap();
    let u_inv_projection = locations.get( "u_inv_projection" ).unwrap().clone().unwrap();
    let u_inv_view = locations.get( "u_inv_view" ).unwrap().clone().unwrap();

    // Define outline parameters ( thickness animated with time )
    let outline_thickness = 30.0;
    let outline_color = [ 1.0, 1.0, 1.0, 1.0 ]; // White outline
    let object_color = [ 1.0, 1.0, 0.0, 1.0 ]; // Grey object

    // Bind the default framebuffer ( render to canvas )
    gl.bind_framebuffer( GL::FRAMEBUFFER, None );

    gl.clear_color( 0.0, 0.0, 0.0, 1.0 );
    gl.clear( GL::COLOR_BUFFER_BIT );

    gl::uniform::upload( gl, Some( u_resolution.clone() ), &[ self.viewport.0 as f32, self.viewport.1 as f32 ] ).unwrap();
    gl::uniform::upload( gl, Some( u_outline_thickness.clone() ), &outline_thickness ).unwrap();
    gl::uniform::upload( gl, Some( u_outline_color.clone() ), &outline_color ).unwrap();
    gl::uniform::upload( gl, Some( u_object_color.clone() ), &object_color ).unwrap();

    gl::uniform::matrix_upload
    (
      gl,
      Some( u_inv_projection.clone() ),
      &self.camera.get_projection_matrix().inverse().unwrap().to_array(),
      true
    )
    .unwrap();
    gl::uniform::matrix_upload
    (
      gl,
      Some( u_inv_view.clone() ),
      &self.camera.get_view_matrix().inverse().unwrap().to_array(),
      true
    )
    .unwrap();

    upload_texture( gl, equirect_map, &u_equirect_map, GL::TEXTURE0 );
    upload_texture( gl, object_fb_color, &u_object_texture, GL::TEXTURE1 );
    upload_texture( gl, object_fb_normal, &u_normal_texture, GL::TEXTURE2 );

    // The final JFA result is in jfa_step_fb_color_0 if num_passes is even, otherwise in jfa_step_fb_color_1
    if num_passes % 2 == 0
    {
      upload_texture( gl, jfa_step_fb_color_0, &u_jfa_step_texture, GL::TEXTURE3 );
    }
    else
    {
      upload_texture( gl, jfa_step_fb_color_1, &u_jfa_step_texture, GL::TEXTURE3 );
    }

    gl.draw_arrays( GL::TRIANGLES, 0, 3 );
  }
}

/// Sets up the application and runs the main rendering loop.
///
/// Initializes the renderer and defines the update/draw function that is called
/// by the `gl::exec_loop::run`.
///
/// # Returns
///
/// A `Result` indicating success or a WebGL error.
async fn run() -> Result< (), gl::WebglError >
{
  let renderer = Renderer::new().await;

  let window = gl::web_sys::window().unwrap();
  let document = window.document().unwrap();

  let gltf_path = "2017_porsche_911_turbo_s_exclusive_series_991.2.glb";
  let gltf = load( &document, gltf_path, &renderer.gl ).await?;
  let scenes = gltf.scenes.clone();
  remove_node_from_scene_by_name( &scenes[ 0 ], "Mesh_0153.rip__0" );
  remove_node_from_scene_by_name( &scenes[ 0 ], "Mesh_0162.rip__0" );
  for node in &scenes[ 0 ].borrow().children
  {
    node.borrow_mut().set_center_to_origin();
    node.borrow_mut().normalize_scale();
    let scale = node.borrow().get_scale();
    node.borrow_mut().set_scale( scale * 30.0 );
  }

  scenes[ 0 ].borrow_mut().update_world_matrix();

  let update_and_draw =
  {
    move | t : f64 |
    {
      renderer.render( scenes[ 0 ].clone(), t );
      true
    }
  };

  gl::exec_loop::run( update_and_draw );

  Ok( () )
}

/// The main entry point of the application.
///
/// Spawns the asynchronous `run` function using `gl::spawn_local` which is
/// suitable for WebAssembly targets in a browser environment.
fn main()
{
  gl::spawn_local( async move { run().await.unwrap() } );
}

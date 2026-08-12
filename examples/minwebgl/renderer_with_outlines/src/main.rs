//! Compares different outline methods for GLTF files.
#![ doc( html_root_url = "https://docs.rs/renderer_with_outlines/latest/renderer_with_outlines/" ) ]
#![ cfg_attr( doc, doc = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/", "readme.md" ) ) ) ]
#![ cfg_attr( not( doc ), doc = "Compares different outline methods for GLTF files" ) ]

use rustc_hash::FxHashMap;
use mingl::F32x4;
use minwebgl as gl;

use gl::
{
  texture::d2::image_upload_from_path,
  GL,
  JsCast,
  web_sys::
  {
    HtmlElement,
    HtmlSelectElement,
    HtmlSpanElement,
    HtmlInputElement,
    HtmlCanvasElement,
    window,
    wasm_bindgen::closure::Closure
  }
};
use renderer::webgl::
{
  loaders::gltf::GLTF,
  geometry::AttributeInfo,
  Camera,
  Renderer,
  TextureInfo,
  Texture,
  Sampler,
  WrappingMode,
  MagFilterMode,
  MinFilterMode,
  post_processing::
  {
    self,
    outline::narrow_outline::NarrowOutlinePass,
    outline::normal_depth_outline::NormalDepthOutlinePass,
    outline::wide_outline::WideOutlinePass,
    GBuffer,
    GBufferAttachment,
    Pass,
    SwapFramebuffer
  }
};
use std::rc::Rc;
use std::cell::RefCell;

/// Creates a new `TextureInfo` struct with a texture loaded from a file.
///
/// This function calls `upload_texture` to load an image, sets up a default `Sampler`
/// with linear filtering and repeat wrapping, and then combines them into a `TextureInfo`
/// struct.
///
/// # Arguments
///
/// * `gl` - The WebGl2RenderingContext.
/// * `image_path` - The path to the image file, relative to the `static/` directory.
///
/// # Returns
///
/// A `TextureInfo` containing the texture data.
fn texture_create
(
  gl : &GL,
  image_path : &str
) -> TextureInfo
{
  let image_path = format!( "static/{image_path}" );
  let texture_id = image_upload_from_path( gl, &image_path, true );

  let sampler = Sampler::former()
  .min_filter( MinFilterMode::Linear )
  .mag_filter( MagFilterMode::Linear )
  .wrap_s( WrappingMode::Repeat )
  .wrap_t( WrappingMode::Repeat )
  .end();

  let texture = Texture::former()
  .target( GL::TEXTURE_2D )
  .source( texture_id )
  .sampler( sampler )
  .end();

  TextureInfo
  {
    texture : Rc::new( RefCell::new( texture ) ),
    uv_position : 0,
  }
}

fn object_colors_generate( object_count : u32 ) -> Vec< F32x4 >
{
  

  ( 0..object_count )
  .map
  (
    | _ |
    {
      F32x4::from_array
      (
        [
          1.0,
          0.0,
          0.0,
          1.0
        ]
      )
    }
  )
  .collect::< Vec< _ > >()
}

fn attributes_get( gltf : &GLTF ) -> Result< FxHashMap< Box< str >, AttributeInfo >, gl::WebglError >
{
  for mesh in &gltf.meshes
  {
    let mesh_ref = mesh.as_ref().borrow();
    if let Some(primitive) = mesh_ref.primitives.first()
    {
      let primitive_ref = primitive.as_ref().borrow();
      return Ok( primitive_ref.geometry.as_ref().borrow().attributes_get().clone() );
    }
  }

  Err( gl::WebglError::MissingDataError( "Primitive" ) )
}

fn html_element_get_by_id( id : &str ) -> HtmlElement
{
  let document = window()
  .unwrap()
  .document()
  .unwrap();
  document.get_element_by_id(id)
  .unwrap()
  .dyn_into::< HtmlElement >()
  .unwrap()
}

fn html_span_element_get_by_id( id : &str ) -> HtmlSpanElement
{
  html_element_get_by_id( id )
  .dyn_into::< HtmlSpanElement >()
  .unwrap()
}

fn html_input_element_get_by_id( id : &str ) -> HtmlInputElement
{
  html_element_get_by_id( id )
  .dyn_into::< HtmlInputElement >()
  .unwrap()
}

/// Initializes the browser context, canvas, and WebGL2 context with the required extensions.
fn context_init() -> Result< ( HtmlCanvasElement, GL ), gl::WebglError >
{
  gl::browser::setup( gl::browser::Config::default() );
  let options = gl::context::ContextOptions::default().antialias( false );

  let canvas = gl::canvas::make()?;
  let gl = gl::context::from_canvas_with( &canvas, options )?;

  let _ = gl.get_extension( "EXT_color_buffer_float" )
  .expect( "Failed to enable EXT_color_buffer_float extension" );

  let _ = gl.get_extension( "EXT_shader_image_load_store" )
  .expect( "Failed to enable EXT_shader_image_load_store  extension" );

  Ok( ( canvas, gl ) )
}

/// Creates the scene camera looking at `center` and binds its controls to the canvas.
fn camera_setup( canvas : &HtmlCanvasElement, center : gl::F32x3 ) -> Camera
{
  let width = canvas.width() as f32;
  let height = canvas.height() as f32;

  let eye = gl::math::F32x3::from( [ 0.0, 1.0, 1.0 ] );
  let up = gl::math::F32x3::from( [ 0.0, 1.0, 0.0 ] );

  let aspect_ratio = width / height;
  let fov = 70.0f32.to_radians();
  let near = 0.01;
  let far = 1_000_000.0;

  let mut camera = Camera::new( eye, up, center, aspect_ratio, fov, near, far );
  camera.window_size_set( [ width, height ].into() );
  camera.controls_bind( canvas );

  camera
}

/// Creates the G-buffer with attachments wired to the mesh attribute buffers.
fn gbuffer_create
(
  gl : &GL,
  canvas : &HtmlCanvasElement,
  attributes : &FxHashMap< Box< str >, AttributeInfo >
) -> Result< GBuffer, gl::WebglError >
{
  let get_buffer = | name | attributes.get( name ).unwrap().buffer.clone();

  let attachments = FxHashMap::from_iter(
    [
      ( GBufferAttachment::Position, vec![ get_buffer( "positions" ) ] ),
      ( GBufferAttachment::Albedo, vec![] ),
      ( GBufferAttachment::Uv1, vec![] ),
      ( GBufferAttachment::Normal, vec![ get_buffer( "normals" ) ] ),
      ( GBufferAttachment::PbrInfo, vec![ get_buffer( "texture_coordinates_2" ) ] ),
      ( GBufferAttachment::ObjectColor, vec![] )
    ]
  );

  GBuffer::new( gl, canvas.width(), canvas.height(), attachments )
}

/// Rendering resources needed to resolve the texture selected in the UI dropdown.
struct SelectTextureContext
{
  gl : Rc< RefCell< GL > >,
  gbuffer : Rc< RefCell< GBuffer > >,
  swap_buffer : Rc< RefCell< SwapFramebuffer > >,
  narrow_outline : Rc< RefCell< NarrowOutlinePass > >,
  normal_depth_outline : Rc< RefCell< NormalDepthOutlinePass > >,
  wide_outline : Rc< RefCell< WideOutlinePass > >,
  outline_thickness : Rc< RefCell< f32 > >,
}

impl SelectTextureContext
{
  /// Creates the context, constructing the three outline passes over the G-buffer.
  fn new
  (
    gl : Rc< RefCell< GL > >,
    gbuffer : Rc< RefCell< GBuffer > >,
    swap_buffer : Rc< RefCell< SwapFramebuffer > >,
    outline_thickness : Rc< RefCell< f32 > >,
    canvas : &HtmlCanvasElement
  ) -> Result< Self, gl::WebglError >
  {
    let narrow_outline = Rc::new
    (
      RefCell::new
      (
        NarrowOutlinePass::new
        (
          &gl.borrow(),
          gbuffer.borrow().texture( GBufferAttachment::Position ),
          gbuffer.borrow().texture( GBufferAttachment::ObjectColor ),
          *outline_thickness.borrow(),
          canvas.width(),
          canvas.height()
        )?
      )
    );

    let normal_depth_outline = Rc::new
    (
      RefCell::new
      (
        NormalDepthOutlinePass::new
        (
          &gl.borrow(),
          gbuffer.borrow().texture( GBufferAttachment::Position ),
          gbuffer.borrow().texture( GBufferAttachment::Normal ),
          gbuffer.borrow().texture( GBufferAttachment::ObjectColor ),
          *outline_thickness.borrow(),
          canvas.width(),
          canvas.height()
        )?
      )
    );

    let wide_outline = Rc::new
    (
      RefCell::new
      (
        WideOutlinePass::new
        (
          &gl.borrow(),
          gbuffer.borrow()
          .texture( GBufferAttachment::ObjectColor ).unwrap(),
          *outline_thickness.borrow(),
          canvas.width(),
          canvas.height()
        )?
      )
    );

    Ok
    (
      Self
      {
        gl,
        gbuffer,
        swap_buffer,
        narrow_outline,
        normal_depth_outline,
        wide_outline,
        outline_thickness
      }
    )
  }

  /// Resolves the texture to display for the given dropdown selection, running outline passes on demand.
  fn select( &self, select_value : &str ) -> Option< gl::web_sys::WebGlTexture >
  {
    let current_outline_thickness = *self.outline_thickness.borrow();

    match select_value
    {
      "position" => self.gbuffer.borrow().texture( GBufferAttachment::Position ),
      "normal" => self.gbuffer.borrow().texture( GBufferAttachment::Normal ),
      "albedo" => self.gbuffer.borrow().texture( GBufferAttachment::Albedo ),
      "object_color" => self.gbuffer.borrow().texture( GBufferAttachment::ObjectColor ),
      "narrow_outline" =>
      {
        self.narrow_outline.borrow_mut()
        .outline_thickness_set( current_outline_thickness );
        self.narrow_outline.borrow_mut()
        .render( &self.gl.borrow(), self.swap_buffer.borrow().input_get(), self.swap_buffer.borrow().output_get() )
        .expect( "Failed to render outline pass" )
      },
      "normal_depth_outline" =>
      {
        self.normal_depth_outline.borrow_mut()
        .outline_thickness_set( current_outline_thickness );
        self.normal_depth_outline.borrow_mut()
        .render( &self.gl.borrow(), self.swap_buffer.borrow().input_get(), self.swap_buffer.borrow().output_get() )
        .expect( "Failed to render outline pass" )
      },
      _ if select_value.starts_with( "wide_outline" ) =>
      {
        if let Some( passes ) = select_value.strip_prefix( "wide_outline" )
        {
          if let Ok( passes ) = passes.parse::< u32 >()
          {
            self.wide_outline.borrow_mut().num_passes_set( passes );
          }
        }

        self.wide_outline.borrow_mut()
        .outline_thickness_set( current_outline_thickness );
        self.wide_outline.borrow_mut()
        .render( &self.gl.borrow(), self.swap_buffer.borrow().input_get(), self.swap_buffer.borrow().output_get() )
        .expect( "Failed to render outline pass" )
      },
      _ => None
    }
  }
}

/// Attaches the change listener that tracks the display dropdown's selected value.
fn select_listener_bind( select_value : Rc< RefCell< String > > )
{
  let select_change_closure = Closure::wrap
  (
    Box::new
    (
    move | event: web_sys::Event |
    {
      let select_element_target = event.target()
      .and_then( | t | t.dyn_into::< HtmlSelectElement >().ok() );
      if let Some( select_elem ) = select_element_target
      {
        ( *select_value.borrow_mut() ).clone_from( &select_elem.value() );
      }
      else
      {
        gl::warn!( "Failed to cast event target to HtmlSelectElement" );
      }
    }
    )
    as Box< dyn FnMut( _ ) >
  );

  let select_element = html_element_get_by_id( "displayOption" );
  let _ = select_element.add_event_listener_with_callback( "change", select_change_closure.as_ref().unchecked_ref() );
  select_change_closure.forget();
}

/// Attaches the input listener that syncs the outline thickness slider with its display span.
fn thickness_slider_bind( outline_thickness : Rc< RefCell< f32 > > )
{
  let outline_thickness_slider_element = html_input_element_get_by_id( "outlineThicknessSlider" );
  let outline_thickness_display_span = html_span_element_get_by_id( "outlineThicknessValue" );

  // Set initial value of the display span
  let () = outline_thickness_display_span.set_text_content( Some( &outline_thickness.borrow().to_string() ) );

  let slider_change_closure =
  Closure::wrap
  (
    Box::new(
      move | event : web_sys::Event |
      {
        let input_element_target = event.target()
        .and_then( | t | t.dyn_into::< HtmlInputElement >().ok() );
        if let Some(input_elem) = input_element_target
        {
          if let Ok( value ) = input_elem.value().parse::<f32>()
          {
            *outline_thickness.borrow_mut() = value;
            let () = outline_thickness_display_span.set_text_content( Some( &value.to_string() ) );
          }
          else
          {
            gl::warn!( "Failed to parse slider value to f32" );
          }
        }
        else
        {
          gl::warn!( "Failed to cast event target to HtmlInputElement" );
        }
      }
    ) as Box< dyn FnMut( _ ) >
  );

  let _ = outline_thickness_slider_element.add_event_listener_with_callback( "input", slider_change_closure.as_ref().unchecked_ref() );
  slider_change_closure.forget();
}

/// Sets up the main 3D scene by loading a GLTF file and configuring objects.
///
/// # Arguments
///
/// * `gl` - The `WebGl2RenderingContext`.
///
/// # Returns
///
/// A `Result` containing the configured `GLTF` scene, or a `gl::WebglError` if loading fails.
async fn scene_setup( gl : &GL ) -> Result< GLTF, gl::WebglError >
{
  let window = web_sys::window().expect( "Can't get window" );
  let document =  window.document().expect( "Can't get document" );

  let gltf_path = "static/2017_porsche_911_turbo_s_exclusive_series_991.2.glb";
  let gltf = renderer::webgl::loaders::gltf::load( &document, gltf_path, gl ).await?;

  let car = gltf.scenes[ 0 ].borrow().children.first()
  .expect( "Scene is empty" ).clone();
  let scale = 10.0;

  car.borrow_mut().scale_set( [ scale; 3 ] );
  car.borrow_mut().local_matrix_update();

  Ok( gltf )
}

async fn app_run() -> Result< (), gl::WebglError >
{
  let ( canvas, gl ) = context_init()?;

  let gltf = scene_setup( &gl ).await.unwrap();
  let scenes = gltf.scenes.clone();

  let scene_bounding_box = scenes[ 0 ].borrow().bounding_box();
  gl::info!( "Scene boudnig box: {scene_bounding_box:?}" );

  let camera = camera_setup( &canvas, scene_bounding_box.center() );

  let renderer = Rc::new
  (
    RefCell::new
    (
      Renderer::new( &gl, canvas.width(), canvas.height(), 4 )?
    )
  );

  let ibl = renderer::webgl::loaders::ibl::load( &gl, "static/environment_maps/pink_sunrise_4k/", None ).await;
  renderer.borrow_mut().ibl_set( ibl );
  let skybox = texture_create( &gl, "environment_maps/equirectangular_maps/pink_sunrise.jpg" );
  renderer.borrow_mut().skybox_set( skybox.texture.borrow().source.clone() );
  let renderer1 = renderer.clone();

  let attributes = attributes_get( &gltf )?;

  gl::info!( "{:?}", attributes.keys() );

  let gbuffer = Rc::new( RefCell::new( gbuffer_create( &gl, &canvas, &attributes )? ) );

  let swap_buffer = Rc::new
  (
    RefCell::new
    (
      SwapFramebuffer::new( &gl, canvas.width(), canvas.height() )
    )
  );

  let sw2 = swap_buffer.clone();

  let tonemapping = post_processing::ToneMappingPass::< post_processing::ToneMappingAces >::new( &gl )?;
  let to_srgb = post_processing::ToSrgbPass::new( &gl, true )?;

  let outline_thickness = Rc::new( RefCell::new( 5.0f32 ) );

  let object_colors = object_colors_generate( gltf.meshes.len() as u32 );

  let gl = Rc::new( RefCell::new( gl ) );
  let gl2 = gl.clone();

  let select_texture = SelectTextureContext::new
  (
    gl,
    gbuffer.clone(),
    swap_buffer,
    outline_thickness.clone(),
    &canvas
  )?;

  let select_value = Rc::new( RefCell::new( String::new() ) );

  select_listener_bind( select_value.clone() );
  thickness_slider_bind( outline_thickness );

  let fps_value = html_span_element_get_by_id( "fpsValue" );
  let mut last_time = 0.0;
  let mut fps = 0;

  // Define the update and draw logic
  let update_and_draw =
  {
    move | t : f64 |
    {
      let time = ( t / 1000.0 ) as f32;

      // Update fps text when a whole second elapsed
      if time as u32 > last_time as u32
      {
        fps_value.set_text_content( Some( &format!( "{fps}" ) ) );
        fps = 0;
      }
      last_time = time;
      fps += 1;

      gbuffer.clone()
      .borrow_mut()
      .render( &gl2.borrow(), &mut scenes[ 0 ].borrow_mut(), Some( &object_colors ), &camera )
      .expect( "Failed to render gbuffer" );

      renderer1.borrow_mut().render( &gl2.borrow(), &mut scenes[ 0 ].borrow_mut(), &camera )
      .expect( "Failed to render" );

      sw2.borrow_mut().reset();
      sw2.borrow_mut().bind( &gl2.borrow() );
      sw2.borrow_mut().input_set( renderer1.borrow().main_texture() );

      if let Some( t ) = select_texture.select( &select_value.borrow() )
      {
        sw2.borrow_mut().bind( &gl2.borrow() );
        sw2.borrow_mut().output_set( Some( t ) );
        sw2.borrow_mut().swap();
      }

      let t = tonemapping.render( &gl2.borrow(), sw2.borrow().input_get(), sw2.borrow().output_get() )
      .expect( "Failed to render tonemapping pass" );

      sw2.borrow_mut().output_set( t );
      sw2.borrow_mut().swap();

      let _t = to_srgb.render( &gl2.borrow(), sw2.borrow().input_get(), sw2.borrow().output_get() )
      .expect( "Failed to render to srgb pass" );

      true
    }
  };

  // Run the render loop
  gl::exec_loop::run( update_and_draw );

  Ok( () )
}

fn main()
{
  gl::spawn_local( async move { app_run().await.unwrap() } );
}

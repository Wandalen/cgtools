//! Compares different outline methods for GLTF files.
#![ doc( html_root_url = "https://docs.rs/renderer_with_outlines/latest/renderer_with_outlines/" ) ]
#![ cfg_attr( doc, doc = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/", "readme.md" ) ) ) ]
#![ cfg_attr( not( doc ), doc = "Compares different outline methods for GLTF files" ) ]

#![ allow( clippy::std_instead_of_core ) ]
#![ allow( clippy::cast_precision_loss ) ]
#![ allow( clippy::too_many_lines ) ]
#![ allow( clippy::needless_pass_by_value ) ]
#![ allow( clippy::std_instead_of_alloc ) ]
#![ allow( clippy::implicit_return ) ]
#![ allow( clippy::let_and_return ) ]
#![ allow( clippy::never_loop ) ]
#![ allow( clippy::default_trait_access ) ]
#![ allow( clippy::uninlined_format_args ) ]
#![ allow( clippy::cast_possible_wrap ) ]
#![ allow( clippy::from_str_radix_10 ) ]
#![ allow( clippy::excessive_precision ) ]
#![ allow( clippy::map_unwrap_or ) ]
#![ allow( clippy::unnecessary_wraps ) ]
#![ allow( clippy::cast_possible_truncation ) ]
#![ allow( clippy::min_ident_chars ) ]
#![ allow( clippy::let_unit_value ) ]
#![ allow( clippy::ignored_unit_patterns ) ]
#![ allow( clippy::cast_sign_loss ) ]

use rustc_hash::FxHashMap;
use mingl::F32x4;
use minwebgl as gl;

use gl::
{
  GL,
  JsCast,
  web_sys::
  {
    WebGlTexture,
    HtmlElement,
    HtmlSelectElement,
    HtmlSpanElement,
    HtmlInputElement,
    window,
    wasm_bindgen::closure::Closure
  }
};
// use rand::Rng;
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

/// Uploads an image from a URL to a WebGL texture.
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
fn upload_texture( gl : &GL, src : &str ) -> WebGlTexture
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
/// An `Option<TextureInfo>` containing the texture data, or `None` if creation fails.
fn create_texture
(
  gl : &GL,
  image_path : &str
) -> Option< TextureInfo >
{
  let image_path = format!( "static/{image_path}" );
  let texture_id = upload_texture( gl, image_path.as_str() );

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

  let texture_info = TextureInfo
  {
    texture : Rc::new( RefCell::new( texture ) ),
    uv_position : 0,
  };

  Some( texture_info )
}

fn generate_object_colors( object_count : u32 ) -> Vec< F32x4 >
{
  // let mut rng = rand::thread_rng();

  // let range = 0.2..1.0;
  let object_colors = ( 0..object_count )
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
        // [
        //   rng.gen_range( range.clone() ),
        //   rng.gen_range( range.clone() ),
        //   rng.gen_range( range.clone() ),
        //   1.0
        // ]
      )
    }
  )
  .collect::< Vec< _ > >();

  object_colors
}

fn get_attributes( gltf : &GLTF ) -> Result< FxHashMap< Box< str >, AttributeInfo >, gl::WebglError >
{
  for mesh in &gltf.meshes
  {
    let mesh_ref = mesh.as_ref().borrow();
    for primitive in &mesh_ref.primitives
    {
      let primitive_ref = primitive.as_ref().borrow();
      return Ok( primitive_ref.geometry.as_ref().borrow().get_attributes().clone() );
    }
  }

  Err( gl::WebglError::MissingDataError( "Primitive" ) )
}

fn get_html_element_by_id( id : &str ) -> HtmlElement
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

fn get_html_span_element_by_id( id : &str ) -> HtmlSpanElement
{
  get_html_element_by_id( id )
  .dyn_into::< HtmlSpanElement >()
  .unwrap()
}

fn get_html_input_element_by_id( id : &str ) -> HtmlInputElement
{
  get_html_element_by_id( id )
  .dyn_into::< HtmlInputElement >()
  .unwrap()
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
async fn setup_scene( gl : &GL ) -> Result< GLTF, gl::WebglError >
{
  let window = web_sys::window().expect( "Can't get window" );
  let document =  window.document().expect( "Can't get document" );

  let gltf_path = "2017_porsche_911_turbo_s_exclusive_series_991.2.glb";
  let gltf = renderer::webgl::loaders::gltf::load( &document, gltf_path, &gl ).await?;

  let car = gltf.scenes[ 0 ].borrow().children.get( 0 )
  .expect( "Scene is empty" ).clone();
  let scale = 10.0;

  car.borrow_mut().set_scale( [ scale; 3 ] );
  car.borrow_mut().update_local_matrix();

  Ok( gltf )
}

async fn run() -> Result< (), gl::WebglError >
{
  gl::browser::setup( Default::default() );
  let options = gl::context::ContexOptions::default().antialias( false );

  let canvas = gl::canvas::make()?;
  let gl = gl::context::from_canvas_with( &canvas, options )?;

  let _ = gl.get_extension( "EXT_color_buffer_float" )
  .expect( "Failed to enable EXT_color_buffer_float extension" );

  let _ = gl.get_extension( "EXT_shader_image_load_store" )
  .expect( "Failed to enable EXT_shader_image_load_store  extension" );

  let width = canvas.width() as f32;
  let height = canvas.height() as f32;

  let gltf = setup_scene( &gl ).await.unwrap();
  let scenes = gltf.scenes.clone();

  let scene_bounding_box = scenes[ 0 ].borrow().bounding_box();
  gl::info!( "Scene boudnig box: {:?}", scene_bounding_box );

  // Camera setup
  let eye = gl::math::F32x3::from( [ 0.0, 1.0, 1.0 ] );
  let up = gl::math::F32x3::from( [ 0.0, 1.0, 0.0 ] );
  let center = scene_bounding_box.center();

  let aspect_ratio = width / height;
  let fov = 70.0f32.to_radians();
  let near = 0.01;
  let far = 1000000.0;

  let mut camera = Camera::new( eye, up, center, aspect_ratio, fov, near, far );
  camera.set_window_size( [ width, height ].into() );
  camera.bind_controls( &canvas );

  let renderer = Rc::new
  (
    RefCell::new
    (
      Renderer::new( &gl, canvas.width(), canvas.height(), 4 )?
    )
  );
  renderer.borrow_mut().set_ibl( renderer::webgl::loaders::ibl::load( &gl, "environment_maps/pink_sunrise_4k/", None ).await );
  let skybox = create_texture( &gl, "environment_maps/equirectangular_maps/pink_sunrise.jpg" ).unwrap();
  renderer.borrow_mut().set_skybox( skybox.texture.borrow().source.clone() );
  let renderer1 = renderer.clone();

  let attributes = get_attributes( &gltf )?;

  gl::info!( "{:?}", attributes.keys() );

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

  let gbuffer = Rc::new
  (
    RefCell::new
    (
      GBuffer::new
      (
        &gl,
        canvas.width(),
        canvas.height(),
        attachments
      )?
    )
  );
  let gbuffer_rc = gbuffer.clone();

  let swap_buffer = Rc::new
  (
    RefCell::new
    (
      SwapFramebuffer::new( &gl, canvas.width(), canvas.height() )
    )
  );

  let sw1 = swap_buffer.clone();
  let sw2 = swap_buffer.clone();

  let tonemapping = post_processing::ToneMappingPass::< post_processing::ToneMappingAces >::new( &gl )?;
  let to_srgb = post_processing::ToSrgbPass::new( &gl, true )?;

  let outline_thickness = Rc::new( RefCell::new( 5.0f32 ) );
  let outline_thickness_1 = outline_thickness.clone();
  let outline_thickness_2 = outline_thickness.clone();

  let narrow_outline = Rc::new
  (
    RefCell::new
    (
      NarrowOutlinePass::new
      (
        &gl,
        gbuffer.borrow().get_texture( GBufferAttachment::Position ),
        gbuffer.borrow().get_texture( GBufferAttachment::ObjectColor ),
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
        &gl,
        gbuffer.borrow().get_texture( GBufferAttachment::Position ),
        gbuffer.borrow().get_texture( GBufferAttachment::Normal ),
        gbuffer.borrow().get_texture( GBufferAttachment::ObjectColor ),
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
        &gl,
        gbuffer.borrow()
        .get_texture( GBufferAttachment::ObjectColor ).unwrap(),
        *outline_thickness.borrow(),
        canvas.width(),
        canvas.height()
      )?
    )
  );

  let object_colors = generate_object_colors( gltf.meshes.len() as u32 );

  let gl = Rc::new( RefCell::new( gl ) );
  let gl1 = gl.clone();
  let gl2 = gl.clone();

  let select_texture = move | select_value : &str |
  {
    let current_outline_thickness = *outline_thickness_1.borrow();

    match select_value
    {
      "position" => gbuffer_rc.borrow().get_texture( GBufferAttachment::Position ),
      "normal" => gbuffer_rc.borrow().get_texture( GBufferAttachment::Normal ),
      "albedo" => gbuffer_rc.borrow().get_texture( GBufferAttachment::Albedo ),
      "object_color" => gbuffer_rc.borrow().get_texture( GBufferAttachment::ObjectColor ),
      "narrow_outline" =>
      {
        let narrow_outline_1 = narrow_outline.clone();
        narrow_outline_1.borrow_mut()
        .set_outline_thickness( current_outline_thickness );
        narrow_outline.clone().borrow_mut()
        .render( &gl1.borrow(), sw1.borrow().get_input(), sw1.borrow().get_output() )
        .expect( "Failed to render outline pass" )
      },
      "normal_depth_outline" =>
      {
        let normal_depth_outline_1 = normal_depth_outline.clone();
        normal_depth_outline_1.borrow_mut()
        .set_outline_thickness( current_outline_thickness );
        normal_depth_outline.clone().borrow_mut()
        .render( &gl1.borrow(), sw1.borrow().get_input(), sw1.borrow().get_output() )
        .expect( "Failed to render outline pass" )
      },
      _ if select_value.starts_with( "wide_outline" ) =>
      {
        let wide_outline_1 = wide_outline.clone();
        if let Some( passes ) = select_value.strip_prefix( "wide_outline" )
        {
          if let Ok( passes ) = u32::from_str_radix( passes, 10 )
          {
            wide_outline_1.borrow_mut().set_num_passes( passes );
          }
        }

        wide_outline_1.borrow_mut()
        .set_outline_thickness( current_outline_thickness );
        let texture = wide_outline_1.borrow_mut()
        .render( &gl1.borrow(), sw1.borrow().get_input(), sw1.borrow().get_output() )
        .expect( "Failed to render outline pass" );

        texture
      },
      _ => None
    }
  };

  let select_value = Rc::new( RefCell::new( String::new() ) );
  let select_value_clone = select_value.clone();

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
        *select_value_clone.borrow_mut() = select_elem.value().to_string();
      }
      else
      {
        gl::warn!( "Failed to cast event target to HtmlSelectElement" );
      }
    }
    )
    as Box< dyn FnMut( _ ) >
  );

  let select_element = get_html_element_by_id( "displayOption" );
  let _ = select_element.add_event_listener_with_callback( "change", select_change_closure.as_ref().unchecked_ref() );
  select_change_closure.forget();

  // --- Slider Event Listener ---
  let outline_thickness_slider_element = get_html_input_element_by_id( "outlineThicknessSlider" );
  let outline_thickness_display_span = get_html_span_element_by_id( "outlineThicknessValue" );

  // Set initial value of the display span
  let _ = outline_thickness_display_span.set_text_content( Some( &outline_thickness.borrow().to_string() ) );

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
            *outline_thickness_2.borrow_mut() = value;
            let _ = outline_thickness_display_span.set_text_content( Some( &value.to_string() ) );
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

  let fps_value = get_html_span_element_by_id( "fpsValue" );
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
        fps_value.set_text_content( Some( &format!( "{}", fps ) ) );
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
      sw2.borrow_mut().set_input( renderer1.borrow().get_main_texture() );

      if let Some( t ) = select_texture( &select_value.borrow() )
      {
        sw2.borrow_mut().bind( &gl2.borrow() );
        sw2.borrow_mut().set_output( Some( t ) );
        sw2.borrow_mut().swap();
      }

      let t = tonemapping.render( &gl2.borrow(), sw2.borrow().get_input(), sw2.borrow().get_output() )
      .expect( "Failed to render tonemapping pass" );

      sw2.borrow_mut().set_output( t );
      sw2.borrow_mut().swap();

      let _t = to_srgb.render( &gl2.borrow(), sw2.borrow().get_input(), sw2.borrow().get_output() )
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
  gl::spawn_local( async move { run().await.unwrap() } );
}

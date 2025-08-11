//! Generate with wfc-image and render tile map on quad.
//!
//! The wfc-image implements the Wave Function Collapse (WFC)
//! algorithm to generate new images based on a sample input image; it works
//! by analyzing the input to learn the local patterns (like tiles or
//! overlapping blocks) and the rules of which patterns can appear next
//! to which, then applies these learned constraints to probabilistically
//! "collapse" possibilities on a grid until a consistent, novel image is
//! generated that shares the structural and textural characteristics of
//! the source.
#![ doc = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/", "readme.md" ) ) ]

use gl::GL;
use image::{ DynamicImage, ImageBuffer, Luma };
use minwebgl as gl;
use ndarray_cg::F32x4x4;
use web_sys::wasm_bindgen::prelude::*;
use minwebgl::dom::create_image_element;
use minwebgl::WebGlVertexArrayObject;
use std::rc::Rc;
use std::cell::RefCell;
use web_sys::{ HtmlInputElement, HtmlButtonElement, FileReader, Event };
use wfc::*;
use wfc_image::{ generate_image, wrap::*, retry::* };
use ndarray_cg::mat3x3h;

/// Tile map size. Length of square map side (a x a).
/// More than 256x256 is very slow.
/// This example can generate only static size square maps.
const SIZE : usize = 50;

/// The size of the patterns to be analyzed from the input tilemap image.
const PATTERN_SIZE : u32 = 3;

/// A struct to hold the application's state.
/// This replaces the global static variables.
struct ApplicationState 
{
  map : Option< Vec< Vec< u8 > > >,
  pattern_image : Option< DynamicImage >,
}

/// Set load callback for an image with `path` and hide it from the UI.
///
/// This function creates an HTML `<img>` element, appends it to the
/// document's body (initially hidden and positioned off-screen),
/// sets its ID, cross-origin, load callback, `src` attributes,
/// to trigger the browser's loading process.
///
/// # Arguments
///
/// * `path`: The path relative to the `/static/` directory, used to construct
/// the image URL and set the element's ID.
/// * `on_load_callback`: A closure that will be invoked with a reference to
/// the loaded `HtmlImageElement` when the browser's `load` event fires for the image.
///
/// # Returns
///
/// Returns `Ok(web_sys::HtmlImageElement)` containing the created image element if
/// successful, or `Err(minwebgl::JsValue)`.
///
/// # Side effects
///
/// * An `<img>` element is created and appended to the document's `<body>`.
/// * The element's ID, styles (`visibility: hidden`, `position: absolute`, etc.), `crossorigin`, `onload` callback, and `src` attributes are set.
/// * The browser starts loading the image asynchronously.
fn load_image
(
  path : &str,
  on_load_callback : Box< dyn Fn( &web_sys::HtmlImageElement ) >,
) -> Result< web_sys::HtmlImageElement, minwebgl::JsValue >
{
  let image = create_image_element( "tileset.png" )?;

  let window = web_sys::window()
  .ok_or_else( || JsValue::from_str( "Failed to get window" ) )?;
  let document = window.document()
  .ok_or_else( || JsValue::from_str( "Failed to get document" ) )?;
  let body = document.body()
  .ok_or_else( || JsValue::from_str( "Failed to get body" ) )?;
  let _ = body.append_child( &image );
  image.set_id( &format!( "{path}" ) );

  let style = image.style();
  let _ = style.set_property( "visibility", "hidden" );
  let _ = style.set_property( "position", "absolute" );
  let _ = style.set_property( "top", "0" );
  let _ = style.set_property( "width", "10px" );
  let _ = style.set_property( "height", "10px" );
  image.set_cross_origin( Some( "anonymous" ) );
  let img = image.clone();
  let on_load_callback : Closure< dyn Fn() > = Closure::new( move || on_load_callback( &img ) );
  image.set_onload
  (
    Some
    (
      on_load_callback
      .as_ref()
      .unchecked_ref()
    )
  );
  on_load_callback.forget();
  let origin = window.location()
  .origin()
  .expect( "Should have an origin" );
  let url = format!( "{origin}/static/{path}" );
  image.set_src( &url );
  Ok( image )
}

/// Handles the `change` event on the file input element.
fn on_input_change
(
  event : Event,
  app_state : Rc< RefCell< ApplicationState > >
)
{
  let Some( target ) = event.target()
  else
  { 
    gl::warn!( "Event target is not present" );
    return;
  };

  let input : HtmlInputElement = target
  .dyn_into()
  .unwrap();

  let Some( file_list ) = input.files()
  else
  { 
    gl::warn!( "Failed to get file list from input" );
    return;
  };
  let Some( file ) = file_list.get( 0 )
  else
  { 
    gl::warn!( "No file selected" );
    return;
  };

  let reader = FileReader::new().unwrap();
  let app_state_clone = Rc::clone( &app_state );
  let onload_callback = Closure::< dyn FnMut( _ ) >::new
  (
    move | _event : Event |
    {
      let reader = _event.target()
      .and_then(| target | target.dyn_into::< FileReader >().ok() );

      if let Some( reader ) = reader
      {
        match reader.result()
        {
          Ok( js_val ) =>
          {
            if let Some( tmx_content ) = js_val.as_string()
            {
              let mut state = app_state_clone.borrow_mut();
              set_pattern( &tmx_content, &mut state );
              generate_map_wfc_image( &mut state );
              render_tile_map( &state );
            }
          },
          _ => gl::warn!( "Can't read input file" )
        }
      }
    }
  );

  reader.set_onload( Some( onload_callback.as_ref().unchecked_ref() ) );
  onload_callback.forget();

  let _ = reader.read_as_text( &file );
}

/// Initializes the file input element for uploading TMX files.
fn input_tilemap_init( app_state: Rc< RefCell< ApplicationState > > ) -> Result< (), JsValue >
{
  let window = web_sys::window().unwrap();
  let document = window.document().unwrap();

  let file_input = document.get_element_by_id( "file-input" )
  .unwrap()
  .dyn_into::< HtmlInputElement >()
  .unwrap();

  let file_input_style = file_input.style();
  let _ = file_input_style.set_property( "position", "absolute" );
  let _ = file_input_style.set_property( "top", "15px" );
  let _ = file_input_style.set_property( "left", "15px" );

  let on_change_callback = Closure::< dyn FnMut( _ ) >::new
  (
    {
      let app_state = Rc::clone( &app_state );
      move | e : Event | on_input_change( e, Rc::clone( &app_state ) )
    }
  );

  file_input.add_event_listener_with_callback( "change", on_change_callback.as_ref().unchecked_ref() )?;
  on_change_callback.forget();

  Ok( () )
}

/// Sets up a button with a click event listener.
fn button_generate_setup(id : &str, top : u32, app_state : Rc< RefCell< ApplicationState > > ) -> Result< (), JsValue >
{
  let window = web_sys::window().unwrap();
  let document = window.document().unwrap();

  let button_element = document.get_element_by_id( id )
  .unwrap()
  .dyn_into::< HtmlButtonElement >()
  .unwrap();

  let button_style = button_element.style();
  let _ = button_style.set_property( "position", "absolute" );
  let _ = button_style.set_property( "top", format!( "{}px", top ).as_str() );
  let _ = button_style.set_property( "left", "15px" );

  let button_callback = Closure::< dyn FnMut( _ ) >::new
  (
    {
      let app_state = Rc::clone(&app_state );
      move | _e : Event |
      {
        let mut state = app_state.borrow_mut();
        generate_map_wfc_image( &mut state );
        render_tile_map( &state );
      }
    }
  );

  let _ = button_element.add_event_listener_with_callback
  (
    "click",
    button_callback.as_ref().unchecked_ref()
  );

  button_callback.forget();

  Ok( () )
}

/// Initializes the application by setting up the browser environment and UI.
fn init()
{
  gl::browser::setup( Default::default() );

  let app_state = Rc::new
  (
    RefCell::new
    (
      ApplicationState 
      {
        map : None,
        pattern_image : None,
      }
    )
  );

  let _ = input_tilemap_init( Rc::clone( &app_state ) );
  let _ = button_generate_setup( "generate-wfc-image", 50, Rc::clone( &app_state ) );

  let window = web_sys::window()
  .expect( "Should have a window" );
  let document = window.document()
  .expect( "Should have a document" );
  let body_style = document.body()
  .unwrap()
  .style();
  let _ = body_style.set_property( "margin", "0" );
  let _ = body_style.set_property( "padding", "0" );
  let _ = body_style.set_property( "overflow", "hidden" );
  let _ = body_style.set_property( "height", "100%" );

  let load = move | _img : &web_sys::HtmlImageElement | {};

  let _ = load_image( "tileset.png", Box::new( load ) );
}

/// Prepares the vertex attributes for rendering a quad.
fn prepare_vertex_attributes() -> WebGlVertexArrayObject
{
  let gl = gl::context::retrieve_or_make()
  .unwrap();

  let position_data: [ f32; 12 ] =
  [
    -1., -1., -1., 1., 1., 1.,
    -1., -1., 1., -1., 1., 1.
  ];

  let uv_data: [ f32; 12 ] =
  [
    0., 1., 0., 0., 1., 0.,
    0., 1., 1., 1., 1., 0.
  ];

  let position_slot = 0;
  let position_buffer = gl::buffer::create( &gl )
  .unwrap();
  gl::buffer::upload( &gl, &position_buffer, &position_data, GL::STATIC_DRAW );

  let uv_slot = 1;
  let uv_buffer = gl::buffer::create( &gl )
  .unwrap();
  gl::buffer::upload( &gl, &uv_buffer, &uv_data, GL::STATIC_DRAW );

  let vao = gl::vao::create( &gl )
  .unwrap();
  gl.bind_vertex_array( Some( &vao ) );
  gl::BufferDescriptor::new::< [ f32; 2 ] >()
  .stride( 2 )
  .offset( 0 )
  .attribute_pointer( &gl, position_slot, &position_buffer )
  .unwrap();
  gl::BufferDescriptor::new::< [ f32; 2 ] >()
  .stride( 2 )
  .offset( 0 )
  .attribute_pointer( &gl, uv_slot, &uv_buffer )
  .unwrap();
  gl.bind_vertex_array( None );

  vao
}

/// Creates a Model-View-Projection (MVP) matrix for the scene.
fn create_mvp() -> F32x4x4
{
  let gl = gl::context::retrieve_or_make()
  .unwrap();

  let width = gl.drawing_buffer_width() as f32;
  let height = gl.drawing_buffer_height() as f32;
  let aspect_ratio = width / height;

  let perspective_matrix = mat3x3h::perspective_rh_gl
  (
    70.0f32.to_radians(),
    aspect_ratio,
    0.1,
    1000.0
  );

  let t = [ 0.0, 0.0, 0.0 ];
  let translate = mat3x3h::translation( t );

  let s = [ 1.95 / 3.0, 1.95 / 3.0, 1.95 / 3.0 ];
  let scale = mat3x3h::scale( s );

  let eye = [ 0.0, 0.0, 1.0 ];
  let up = [ 0.0, 1.0, 0.0 ];
  let center = [ 0., 0., 0. ];
  let view_matrix = mat3x3h::look_at_rh( eye, center, up );

  perspective_matrix * view_matrix * translate * scale
}

/// Binds an RGBA texture from an image `id` to a specified `texture_id` slot.
fn prepare_texture_array( id : &str, texture_id : u32 ) -> Option< web_sys::WebGlTexture >
{
  let gl = gl::context::retrieve_or_make()
  .unwrap();

  let window = web_sys::window()
  .expect( "Should have a window" );
  let document = window.document()
  .expect( "Should have a document" );
  let img = document.get_element_by_id( id )?;
  let img = img.dyn_into::< web_sys::HtmlImageElement >()
  .unwrap();

  let width = img.natural_width();
  let layers = img.natural_height() / width;
  let height = img.natural_height() / layers;

  let texture_array = gl.create_texture();
  gl.active_texture( texture_id );
  gl.bind_texture( GL::TEXTURE_2D_ARRAY, texture_array.as_ref() );

  let _ = gl.tex_image_3d_with_html_image_element
  (
    GL::TEXTURE_2D_ARRAY,
    0,
    GL::RGBA as i32,
    width as i32,
    height as i32,
    layers as i32,
    0,
    GL::RGBA,
    GL::UNSIGNED_BYTE,
    &img,
  );

  gl.tex_parameteri
  (
    GL::TEXTURE_2D_ARRAY,
    GL::TEXTURE_MIN_FILTER,
    GL::NEAREST as i32,
  );
  gl.tex_parameteri
  (
    GL::TEXTURE_2D_ARRAY,
    GL::TEXTURE_MAG_FILTER,
    GL::NEAREST as i32,
  );

  gl.tex_parameteri
  (
    GL::TEXTURE_2D_ARRAY,
    GL::TEXTURE_WRAP_S,
    GL::CLAMP_TO_EDGE as i32,
  );
  gl.tex_parameteri
  (
    GL::TEXTURE_2D_ARRAY,
    GL::TEXTURE_WRAP_T,
    GL::CLAMP_TO_EDGE as i32,
  );

  texture_array
}

/// Binds an R8UI texture from `data` with `size` to a specified `texture_id` slot.
fn prepare_texture1u
(
  data: &[ u8 ],
  size: ( i32, i32 ),
  texture_id: u32,
)
{
  let gl = gl::context::retrieve_or_make()
  .unwrap();

  let texture = gl.create_texture();
  gl.active_texture( texture_id );
  gl.bind_texture( GL::TEXTURE_2D, texture.as_ref() );
  gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array
  (
    GL::TEXTURE_2D,
    0,
    GL::R8UI as i32,
    size.0,
    size.1,
    0,
    GL::RED_INTEGER,
    GL::UNSIGNED_BYTE,
    Some( data ),
  )
  .expect( "Can't load an image" );
  gl.pixel_storei( GL::UNPACK_ALIGNMENT, 1 );

  gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_MIN_FILTER, GL::NEAREST as i32 );
  gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_MAG_FILTER, GL::NEAREST as i32 );

  gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_WRAP_S, GL::CLAMP_TO_EDGE as i32 );
  gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_WRAP_T, GL::CLAMP_TO_EDGE as i32 );
}

/// Renders the tile map on the quad.
fn render_tile_map(app_state: &ApplicationState)
{
  let Some( ref map ) = app_state.map
  else
  {
    return;
  };
  if map.is_empty() || map[ 0 ].is_empty()
  {
    return;
  };

  let gl = gl::context::retrieve_or_make()
  .unwrap();

  let vertex_shader_src = include_str!( "../shaders/shader.vert" );
  let fragment_shader_src = include_str!( "../shaders/shader.frag" );
  let program = gl::ProgramFromSources::new( vertex_shader_src, fragment_shader_src )
  .compile_and_link( &gl )
  .unwrap();
  gl.use_program( Some( &program ) );

  let mvp = create_mvp();
  let mvp_location = gl.get_uniform_location( &program, "mvp" );

  gl::uniform::matrix_upload( &gl, mvp_location, mvp.raw_slice(), false )
  .unwrap();

  let vao = prepare_vertex_attributes();
  gl.bind_vertex_array( Some( &vao ) );
  prepare_texture_array( "tileset.png", GL::TEXTURE0 );

  let size = ( map[ 0 ].len() as i32, map.len() as i32 );
  let data = map.iter()
  .cloned()
  .flatten()
  .collect::< Vec< u8 > >();

  prepare_texture1u( &data, size, GL::TEXTURE1 );

  let tiles_location = gl.get_uniform_location( &program, "tiles_sampler" );
  let map_location = gl.get_uniform_location( &program, "map_sampler" );

  gl.uniform1i( tiles_location.as_ref(), 0 );
  gl.uniform1i( map_location.as_ref(), 1 );

  let texel_size = [ 1.0 / size.0 as f32, 1.0 / size.1 as f32 ];
  let texel_size_location = gl.get_uniform_location( &program, "texel_size" );
  let _ = gl::uniform::upload( &gl, texel_size_location, texel_size.as_slice() );

  gl.draw_arrays( GL::TRIANGLES, 0, 3 * 2 );
  gl.bind_vertex_array( None );
}

/// Parses and sets the reference pattern for generating the tilemap from the content of a TMX file.
fn set_pattern( tmx_content : &str, app_state : &mut ApplicationState )
{
  let elem : xml::Element = tmx_content.parse().unwrap();

  let layer = elem.get_child( "layer", None ).unwrap();
  let width = layer.attributes.get( &( "width".to_string(), None ) )
  .clone()
  .unwrap()
  .parse::< u32 >()
  .unwrap();
  let height = layer.attributes.get( &( "height".to_string(), None ) )
  .clone()
  .unwrap()
  .parse::< u32 >()
  .unwrap();
  let data = layer.get_children( "data", None )
  .filter(| ch | ch.attributes.get( &( "encoding".to_string(), None ) ) == Some( &"csv".to_string() ) )
  .next()
  .unwrap();

  let pattern_raw = data.content_str().split( "," )
  .map( | tile | tile.trim().parse::< u8 >().unwrap().saturating_sub( 1 ) )
  .collect::< Vec< _ > >();

  let pattern_buf : ImageBuffer< Luma< u8 >, Vec< u8 > > =
  ImageBuffer::from_vec( width, height, pattern_raw )
  .unwrap();
  let pattern_img = DynamicImage::ImageLuma8( pattern_buf );

  app_state.pattern_image = Some( pattern_img );
}

/// Generates a new tile map using the WFC algorithm with the loaded pattern image.
fn generate_map_wfc_image( app_state : &mut ApplicationState )
{
  let Some( ref pattern_img ) = app_state.pattern_image
  else
  {
    return;
  };

  let Ok( map_img ) = generate_image
  (
    pattern_img,
    std::num::NonZero::new( PATTERN_SIZE ).unwrap(),
    Size::try_new( SIZE as u32, SIZE as u32 ).unwrap(),
    &wfc::orientation::ALL,
    WrapXY,
    ForbidNothing,
    NumTimes( 1 )
  )
  else
  {
    return;
  };

  let map_raw : Vec<u8> = map_img.to_luma8().into_raw();
  let map = map_raw.chunks( SIZE as usize )
  .map( | row | row.to_vec() )
  .collect::< Vec< Vec< _ > > >();

  app_state.map = Some( map );
}

/// Runs the main application logic.
fn run()
{
  init();
}

/// The main entry point of the Rust program.
fn main()
{
  run()
}
use minwebgl as gl;
use gl::{ GL, BufferDescriptor };
use renderer::webgl::{ AttributeInfo, Geometry };
use tiles_tools::coordinates::hexagonal::Coordinate;
use std::{ cell::RefCell, fmt::Debug, rc::Rc };
use strum::{ AsRefStr, EnumIter, EnumString, VariantArray };
use web_sys::
{
  wasm_bindgen::prelude::*,
  WebGlTexture,
  HtmlImageElement,
  HtmlButtonElement,
  HtmlOptionElement,
  HtmlSelectElement,
};
use crate::core_game;

#[ derive( Debug, Clone, Copy, AsRefStr, EnumIter, EnumString, VariantArray, PartialEq, Eq ) ]
pub enum EditMode
{
  EditTiles,
  EditRivers,
}

pub struct Texture
{
  pub size : Rc< RefCell< gl::U32x2 > >,
  pub texture : Option< web_sys::WebGlTexture >
}

pub fn setup_select< 'a, I >( document : &web_sys::Document, id : &str, variants : I ) -> HtmlSelectElement
where
  I : Iterator< Item = &'a String >
{
  let select_element = document.get_element_by_id( id ).unwrap();
  let select_element = select_element.dyn_into::< HtmlSelectElement >().unwrap();
  for variant in variants
  {
    let option_element = document.create_element( "option" )
    .unwrap()
    .dyn_into::< HtmlOptionElement >()
    .unwrap();

    option_element.set_value( variant );
    option_element.set_text( variant );
    select_element.add_with_html_option_element( &option_element ).unwrap();
  }
  return select_element;
}

pub fn setup_download_button
(
  document : &web_sys::Document,
  map : Rc< RefCell< core_game::Map > >
)
{
  let button = document.get_element_by_id( "download" )
  .unwrap()
  .dyn_into::< HtmlButtonElement >()
  .unwrap();

  let onclick : Closure< dyn Fn() > = Closure::new
  ({
    move || download_map( &map.borrow() )
  });

  button.set_onclick( Some( onclick.as_ref().unchecked_ref() ) );
  onclick.forget();
}

fn download_map( map : &core_game::Map )
{
  let json = serde_json::to_string( map ).unwrap();
  let array = web_sys::js_sys::Array::new();
  array.push( &JsValue::from_str( &json ) );

  let url = gl::blob::create_blob( array, "application/json" ).unwrap();

  let window = web_sys::window().unwrap();
  let document = window.document().unwrap();

  let anchor = document.create_element( "a" )
  .unwrap()
  .dyn_into::< web_sys::HtmlAnchorElement >()
  .unwrap();

  let [ q, r ] = calculate_map_size( map );
  let file_name = format!( "map_{q}x{r}.json" );
  anchor.set_href( &url );
  anchor.set_download( &file_name );
  anchor.click();
}

pub fn setup_drop_zone
(
  document : &web_sys::Document,
  map_json : Rc< RefCell< Option< String > > >
)
{
  let element = document.get_element_by_id( "drop-zone" ).unwrap();

  let prevent_default = Closure::< dyn Fn( _ ) >::new
  (
    | e : web_sys::Event |
    {
      e.prevent_default();
      e.stop_propagation();
    }
  );

  element.add_event_listener_with_callback
  (
    "dragover",
    prevent_default.as_ref().unchecked_ref()
  ).unwrap();
  element.add_event_listener_with_callback
  (
    "dragenter",
    prevent_default.as_ref().unchecked_ref()
  ).unwrap();
  prevent_default.forget();

  let drop_handler = Closure::< dyn Fn( _ ) >::new
  (
    move | e : web_sys::DragEvent |
    {
      e.prevent_default();
      e.stop_propagation();

      if let Some( file ) = e.data_transfer()
      .and_then( | dt | dt.files() )
      .and_then( | files | files.get( 0 ) )
      {
        upload_json_map( file, map_json.clone() );
      }
    }
  );

  element.add_event_listener_with_callback
  (
    "drop",
    drop_handler.as_ref().unchecked_ref()
  ).unwrap();
  drop_handler.forget();
}

fn upload_json_map( file : web_sys::File, map_json : Rc< RefCell< Option< String > > > )
{
  let reader = web_sys::FileReader::new().unwrap();
  reader.read_as_text( &file ).unwrap();

  let onload = Closure::< dyn Fn( _ ) >::new
  ({
    let reader = reader.clone();
    move | _ : web_sys::Event |
    {
      let Ok( result ) = reader.result() else
      {
        return;
      };
      let Some( text ) = result.as_string() else
      {
        return;
      };

      *map_json.borrow_mut() = Some( text );
    }
  });

  reader.set_onloadend( Some( onload.as_ref().unchecked_ref() ) );
  onload.forget();
}

pub fn load_texture
(
  gl : &GL,
  document : &web_sys::Document,
  src : &str,
) -> ( Option< WebGlTexture >, Rc< RefCell< gl::U32x2 > > )
{
  let img = document.create_element( "img" )
  .unwrap()
  .dyn_into::< HtmlImageElement >()
  .unwrap();
  img.style().set_property( "display", "none" ).unwrap();
  let texture = gl.create_texture();
  let size = Rc::new( RefCell::new( gl::U32x2::default() ) );

  let on_load : Closure< dyn Fn() > = Closure::new
  ({
    let gl = gl.clone();
    let img = img.clone();
    let texture = texture.clone();
    let size = size.clone();
    move ||
    {
      let width = img.natural_width();
      let height = img.natural_height();
      *size.borrow_mut() = [ width, height ].into();
      gl::texture::d2::upload( &gl, texture.as_ref(), &img );
      gl::texture::d2::filter_linear( &gl );
      gl::texture::d2::wrap_clamp( &gl );
      img.remove();
    }
  });

  img.set_onload( Some( on_load.as_ref().unchecked_ref() ) );
  img.set_src( &src );
  on_load.forget();

  ( texture, size )
}

pub fn create_hexagon_geometry( gl : &GL ) -> Result< Geometry, gl::WebglError >
{
  let positions = tiles_tools::geometry::hexagon_triangles();

  let position_buffer = gl::buffer::create( &gl )?;
  gl::buffer::upload( &gl, &position_buffer, positions.as_slice(), GL::STATIC_DRAW );
  let pos_info = AttributeInfo
  {
    slot : 0,
    buffer : position_buffer,
    descriptor : BufferDescriptor::new::< [ f32; 2 ] >(),
    bounding_box : Default::default(),
  };

  let mut geometry = Geometry::new( &gl )?;
  geometry.vertex_count = positions.len() as u32 / 2;
  geometry.add_attribute( &gl, "position", pos_info, false )?;

  gl.bind_vertex_array( None );

  Ok( geometry )
}

pub fn create_line_geometry( gl : &GL ) -> Result< Geometry, gl::WebglError >
{
  let positions = tiles_tools::geometry::hexagon_lines();
  let position_buffer = gl::buffer::create( &gl )?;
  gl::buffer::upload( &gl, &position_buffer, positions.as_slice(), GL::STATIC_DRAW );
  let pos_info = AttributeInfo
  {
    slot : 0,
    buffer : position_buffer,
    descriptor : BufferDescriptor::new::< [ f32; 2 ] >(),
    bounding_box : Default::default(),
  };

  let mut geometry = Geometry::new( &gl )?;
  geometry.draw_mode = GL::LINES;
  geometry.vertex_count = positions.len() as u32 / 2;
  geometry.add_attribute( &gl, "position", pos_info, false )?;
  gl.bind_vertex_array( None );

  Ok( geometry )
}

pub fn hexagon_shader( gl : &GL, player_count : usize ) -> Result< gl::shader::Program, gl::WebglError >
{
  let vert = include_str!( "../shaders/main.vert" );
  let frag = include_str!( "../shaders/main.frag" );
  let frag = frag.replace( "#define PLAYER_COUNT 1", &format!( "#define PLAYER_COUNT {player_count}" ) );
  gl::shader::Program::new( gl.clone(), vert, &frag )
}

pub fn line_shader( gl : &GL ) -> Result< gl::shader::Program, gl::WebglError >
{
  let vert = include_str!( "../shaders/main.vert" );
  let frag = include_str!( "../shaders/line.frag" );
  gl::shader::Program::new( gl.clone(), vert, frag )
}

pub fn river_shader( gl : &GL ) -> Result< gl::shader::Program, gl::WebglError >
{
  let vert = include_str!( "../shaders/river.vert" );
  let frag = include_str!( "../shaders/river.frag" );
  gl::shader::Program::new( gl.clone(), vert, frag )
}

pub fn sprite_shader( gl : &GL ) -> Result< gl::shader::Program, gl::WebglError >
{
  let vert = include_str!( "../shaders/sprite.vert" );
  let frag = include_str!( "../shaders/sprite.frag" );
  gl::shader::Program::new( gl.clone(), vert, frag )
}

pub fn river_edge_shader( gl : &GL ) -> Result< gl::shader::Program, gl::WebglError >
{
  let vert = include_str!( "../shaders/river_edge.vert" );
  let frag = include_str!( "../shaders/river.frag" );
  gl::shader::Program::new( gl.clone(), vert, frag )
}

pub fn calculate_map_size( map : &crate::core_game::Map ) -> [ i64; 2 ]
{
  let mut min_q = None;
  let mut max_q = None;
  let mut min_r = None;
  let mut max_r = None;

  map.tiles.keys().for_each
  (
  | Coordinate { q, r, ..  } |
    {
      let min_q = min_q.get_or_insert( *q );
      *min_q = ( *min_q ).min( *q );

      let max_q = max_q.get_or_insert( *q );
      *max_q = ( *max_q ).max( *q );

      let min_r = min_r.get_or_insert( *r );
      *min_r = ( *min_r ).min( *r );

      let max_r = max_r.get_or_insert( *r );
      *max_r = ( *max_r ).max( *r );
    }
  );

  let min_q = min_q.unwrap_or_default() as i64;
  let max_q = max_q.map_or( 0, | inner | inner + 1 ) as i64;
  let min_r = min_r.unwrap_or_default() as i64;
  let max_r = max_r.map_or( 0, | inner | inner + 1 ) as i64;

  [ max_q - min_q, max_r - min_r ]
}

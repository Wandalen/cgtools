use minwebgl as gl;
use gl::GL;
use std::{ cell::RefCell, fmt::Debug, rc::Rc };
use strum::IntoEnumIterator;
use serde::Deserialize;
use web_sys::
{
  wasm_bindgen::prelude::*,
  WebGlTexture,
  HtmlImageElement,
  HtmlButtonElement,
  HtmlOptionElement,
  HtmlSelectElement,
};
use crate::{ blob, Map };

#[ derive( Debug, Deserialize ) ]
pub struct SubTexture
{
  #[ serde( rename = "@name" ) ] // @ prefix indicates an XML attribute
  pub name : String,
  #[ serde( rename = "@x" ) ]
  pub x : u32,
  #[ serde( rename = "@y" ) ]
  pub y : u32,
  #[ serde( rename = "@width" ) ]
  pub width : u32,
  #[ serde( rename = "@height" ) ]
  pub height : u32,
}

// Represents the root <TextureAtlas> element
#[ derive( Debug, Deserialize ) ]
#[ serde( rename = "TextureAtlas" ) ] // Maps to the XML element name
pub struct TextureAtlas
{
  #[ serde( rename = "@imagePath" ) ]
  pub image_path : String,
  #[ serde( rename = "SubTexture", default ) ]
  pub sub_textures : Vec< SubTexture >,
}

pub fn setup_select< T >( document : &web_sys::Document, id : &str ) -> HtmlSelectElement
where
  T : IntoEnumIterator + AsRef< str >
{
  let select_element = document.get_element_by_id( id ).unwrap();
  let select_element = select_element.dyn_into::< HtmlSelectElement >().unwrap();
  for variant in T::iter()
  {
    let option_value = variant.as_ref();
    let option_element = document.create_element( "option" )
    .unwrap()
    .dyn_into::< HtmlOptionElement >()
    .unwrap();

    option_element.set_value( option_value );
    option_element.set_text( option_value );
    select_element.add_with_html_option_element( &option_element ).unwrap();
  }
  return select_element;
}

pub fn get_variant< T >( select_element : &HtmlSelectElement ) -> T
where T : std::str::FromStr< Err : Debug >
{
  let value = select_element.value();
  let variant = T::from_str( &value ).unwrap();
  variant
}

pub fn setup_download_button
(
  document : &web_sys::Document,
  map : Rc< RefCell< Map > >
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

fn download_map( map : &Map )
{
  let json = map.to_json();
  let array = web_sys::js_sys::Array::new();
  array.push( &JsValue::from_str( &json ) );

  let url = blob::create_blob( array, "application/json" ).unwrap();

  let window = web_sys::window().unwrap();
  let document = window.document().unwrap();

  let anchor = document.create_element( "a" )
  .unwrap()
  .dyn_into::< web_sys::HtmlAnchorElement >()
  .unwrap();

  anchor.set_href( &url );
  anchor.set_download( "data.json" );
  anchor.click();
}

pub fn setup_drop_zone
(
  document : &web_sys::Document,
  map : Rc< RefCell< Map > >
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
        read_json_file( file, map.clone() );
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

fn read_json_file( file : web_sys::File, map : Rc< RefCell< Map > > )
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

      *map.borrow_mut() = Map::from_json( &text );
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

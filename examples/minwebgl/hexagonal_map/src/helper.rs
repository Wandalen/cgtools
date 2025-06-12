use minwebgl as gl;
use tiles_tools::coordinates::hexagonal;
use hexagonal::Coordinate;
use strum::{ AsRefStr, EnumIter, EnumString, IntoEnumIterator as _ };
use std::{ cell::RefCell, collections::HashMap, rc::Rc, str::FromStr as _ };
use serde::{ Deserialize, Serialize };
use web_sys::
{
  wasm_bindgen::prelude::*,
  HtmlButtonElement,
  HtmlOptionElement,
  HtmlSelectElement,
};

use crate::blob;

type Axial = Coordinate< hexagonal::Axial, hexagonal::Flat >;

#[ derive( Debug, Serialize, Deserialize, Clone, Copy ) ]
pub struct Tile
{
  pub value : TileValue,
  // TODO: New type
  pub owner : u8,
}

#[ derive( Debug, Clone, Copy, AsRefStr, EnumIter, EnumString, Serialize, Deserialize ) ]
pub enum TileValue
{
  Empty,
  Capital,
  Castle,
  Trees,
  Stones,
}

impl TileValue
{
  pub fn to_asset< 'a >( &self, atlas : &'a TextureAtlas ) -> &'a SubTexture
  {
    let sprite_name = match self
    {
      TileValue::Empty => "grass_05.png",
      TileValue::Capital => "medieval_smallCastle.png",
      TileValue::Trees => "grass_10.png",
      TileValue::Stones => "grass_15.png",
      TileValue::Castle => "medieval_largeCastle.png",
    };
    atlas.sub_textures.iter().find( | item | item.name == sprite_name ).unwrap()
  }
}

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


pub fn setup_select_element( document : &web_sys::Document ) -> HtmlSelectElement
{
  let select = document.get_element_by_id( "tile" ).unwrap();
  let select_element = select.dyn_into::< HtmlSelectElement >().unwrap();
  for variant in TileValue::iter()
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

pub fn get_variant( select_element : &HtmlSelectElement ) -> TileValue
{
  let value = select_element.value();
  let variant = TileValue::from_str( &value ).unwrap();
  variant
}

pub fn setup_download_button
(
  document : &web_sys::Document,
  map : Rc< RefCell< HashMap::< Axial, Tile > > >
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

fn download_map( map : &HashMap::< Axial, Tile > )
{
  let map = map.to_owned().into_iter().collect::< Vec< _ > >();
  let json = serde_json::to_string( &map ).unwrap();
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
  map : Rc< RefCell< HashMap::< Axial, Tile > > >
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

fn read_json_file( file : web_sys::File, map : Rc< RefCell< HashMap::< Axial, Tile > > > )
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

      match serde_json::from_str::< Vec::< ( Axial, Tile ) > >( &text )
      {
        Ok( v ) =>
        {
          *map.borrow_mut() = HashMap::from_iter
          (
            v.into_iter()
          )
        },
        Err( e ) => gl::error!( "{e:?}" ),
      }
    }
  });

  reader.set_onloadend( Some( onload.as_ref().unchecked_ref() ) );
  onload.forget();
}

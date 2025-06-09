mod blob;
mod sprite;

use minwebgl as gl;
use browser_input::{ keyboard::KeyboardKey, mouse::MouseButton, Event, EventType };
use std::{ cell::RefCell, collections::HashMap, rc::Rc, str::FromStr as _ };
use strum::{ AsRefStr, EnumIter, IntoEnumIterator, EnumString };
use tiles_tools::coordinates::{ hexagonal, pixel::Pixel };
use renderer::webgl::{ AttributeInfo, Geometry };
use serde::{ Deserialize, Serialize };
use hexagonal::Coordinate;
use gl::
{
  JsCast as _,
  F32x2,
  I32x2,
  Vector,
  GL,
  BufferDescriptor,
  geometry::BoundingBox
};
use web_sys::
{
  wasm_bindgen::prelude::*,
  HtmlButtonElement,
  HtmlCanvasElement,
  HtmlImageElement,
  HtmlOptionElement,
  HtmlSelectElement,
  WebGlTexture,
};

type Axial = Coordinate< hexagonal::Axial, hexagonal::Flat >;

#[ derive( Debug, Serialize, Deserialize ) ]
struct Tile
{
  value : TileValue,
  // TODO: New type
  owner : u8,
}

#[ derive( Debug, Clone, Copy, AsRefStr, EnumIter, EnumString, Serialize, Deserialize ) ]
enum TileValue
{
  Empty,
  Capital,
  Trees,
  Stones,
  Castle,
}

impl TileValue
{
  fn to_asset< 'a >( &self, atlas : &'a TextureAtlas ) -> &'a SubTexture
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

fn main()
{
  gl::spawn_local( async move { run().await.unwrap() } );
}

async fn run() -> Result< (), gl::WebglError >
{
  gl::browser::setup( Default::default() );
  let window = web_sys::window().unwrap();
  let document = window.document().unwrap();
  let fwidth = window.inner_width().unwrap().as_f64().unwrap();
  let fheight = window.inner_height().unwrap().as_f64().unwrap();
  let dpr = window.device_pixel_ratio();
  let gl = gl::context::retrieve_or_make().unwrap();

  let canvas = gl.canvas().unwrap().dyn_into::< HtmlCanvasElement >().unwrap();
  let width = ( fwidth * dpr ) as i32;
  let height = ( fheight * dpr ) as i32;
  canvas.set_width( width as u32 );
  canvas.set_height( height as u32 );
  browser_input::prevent_rightclick( canvas.clone().dyn_into().unwrap() );

  gl.clear_color( 0.0, 0.15, 0.5, 1.0 );
  gl.viewport( 0, 0, width, height );

  let hexagon = create_hexagon_geometry( &gl )?;
  let outline = create_line_geometry( &gl )?;

  hexagon.bind( &gl );
  let main_shader = create_shader( &gl )?;
  let line_shader = create_line_shader( &gl )?;

  let sprite_sheet_path = "static/kenney_hexagon_pack/Spritesheets/hexagonAll_sheet.png";
  let sprite_sheet = load_sprite_sheet( &gl, &document, sprite_sheet_path );
  gl.bind_texture( GL::TEXTURE_2D, sprite_sheet.as_ref() );

  let atlas_size = 2048.0f32;
  let atlas = gl::file::load
  (
    "kenney_hexagon_pack/Spritesheets/hexagonAll_sheet.xml"
  ).await.unwrap();
  let atlas = String::from_utf8( atlas ).unwrap();
  let atlas : TextureAtlas = quick_xml::de::from_str( &atlas ).unwrap();

  let tile_select_element = setup_select_element( &document );
  let player_select = document.get_element_by_id( "player" ).unwrap();
  let player_select = player_select.dyn_into::< HtmlSelectElement >().unwrap();

  let map = Rc::new( RefCell::new( HashMap::< Axial, Tile >::new() ) );
  setup_download_button( &document, map.clone() );
  setup_drop_zone( &document, map.clone() );

  let mut input = browser_input::Input::new
  (
    Some( canvas.dyn_into().unwrap() ),
    browser_input::CLIENT,
  );
  let dpr = dpr as f32;
  let mut zoom = 0.0625;
  let zoom_factor = 0.75;
  let inv_canvas_size = F32x2::new( 1.0 / width as f32, 1.0 / height as f32 );
  let aspect = F32x2::new( 1.0, width as f32 / height as f32 );
  let mut camera_pos = F32x2::default();
  let mut last_pointer_pos : Option< I32x2 > = None;

  let update = move | _ |
  {
    input.update_state();
    let pointer_pos = input.pointer_position();

    for Event { event_type, .. } in input.event_queue().iter()
    {
      if let EventType::Wheel( Vector( [ _, delta, _ ] ) ) = event_type
      {
        if delta.is_sign_negative()
        {
          zoom /= zoom_factor;
        }
        else
        {
          zoom *= zoom_factor;
        }
      }
    }

    let pointer_pos = screen_to_world( pointer_pos, inv_canvas_size, dpr, aspect, zoom );
    let mut hexagonal_pos = pointer_pos - camera_pos;
    hexagonal_pos[ 1 ] = -hexagonal_pos[ 1 ];
    let hexagonal_pos : Pixel = hexagonal_pos.into();
    let coordinate = hexagonal_pos.into();

    if input.is_key_down( KeyboardKey::Space )
    && input.is_button_down( MouseButton::Main )
    {
      if let Some( last_pointer_pos ) = last_pointer_pos
      {
        let last_pointer_pos = screen_to_world
        (
          last_pointer_pos,
          inv_canvas_size,
          dpr,
          aspect,
          zoom
        );
        let movement = pointer_pos - last_pointer_pos;
        camera_pos += movement;
      }
    }
    else if input.is_button_down( MouseButton::Main )
    {
      let variant = get_variant( &tile_select_element );
      let owner : u8 = player_select.value().parse().unwrap();
      map.borrow_mut().insert( coordinate, Tile { value : variant, owner } );
    }
    else if input.is_button_down( MouseButton::Secondary )
    {
      map.borrow_mut().remove( &coordinate );
    }

    last_pointer_pos = Some( input.pointer_position() );

    input.clear_events();

    gl.clear( GL::COLOR_BUFFER_BIT );

    main_shader.activate();
    main_shader.uniform_upload( "u_scale", ( zoom * aspect ).as_slice() );
    main_shader.uniform_upload( "u_camera_pos", camera_pos.as_slice() );
    line_shader.activate();
    line_shader.uniform_upload( "u_scale", ( zoom * aspect ).as_slice() );
    line_shader.uniform_upload( "u_camera_pos", camera_pos.as_slice() );
    for ( coord, Tile { value, owner } ) in map.borrow().iter()
    {
      let axial : Axial = ( *coord ).into();
      let sprite = value.to_asset( &atlas );

      let sprite_offset = F32x2::from_array( [ sprite.x as f32, sprite.y as f32 ] ) / atlas_size;
      let sprite_size = F32x2::from_array( [ sprite.width as f32, sprite.height as f32 ] ) / atlas_size;

      let mut position : Pixel = axial.into();
      position.data[ 1 ] = -position.data[ 1 ];

      hexagon.bind( &gl );
      main_shader.activate();
      gl.vertex_attrib2fv_with_f32_array( 1, &position.data );
      gl.vertex_attrib2fv_with_f32_array( 3, sprite_offset.as_slice() );
      gl.vertex_attrib2fv_with_f32_array( 4, sprite_size.as_slice() );
      gl.vertex_attrib1f( 5, *owner as f32 );
      hexagon.draw( &gl );

      outline.bind( &gl );
      line_shader.activate();
      gl.vertex_attrib2fv_with_f32_array( 1, &position.data );
      outline.draw( &gl );
    }

    true
  };
  gl::exec_loop::run( update );

  Ok( () )
}

fn screen_to_world
(
  screen : I32x2,
  inv_canvas_size : F32x2,
  dpr : f32,
  aspect : F32x2,
  zoom : f32
) -> F32x2
{
  let Vector ( [ x, y ] ) = screen;
  let screenf32 = F32x2::new( x as f32, y as f32 );
  let mut world = ( screenf32 * dpr * inv_canvas_size - 0.5 ) * 2.0 / ( zoom * aspect );
  world[ 1 ] = -world[ 1 ];
  world
}

fn create_hexagon_geometry( gl : &GL ) -> Result< Geometry, minwebgl::WebglError >
{
  let positions = tiles_tools::geometry::hexagon_triangles();
  let tex_coords = tex_coords( &positions );

  let position_buffer = gl::buffer::create( &gl )?;
  gl::buffer::upload( &gl, &position_buffer, positions.as_slice(), GL::STATIC_DRAW );
  let pos_info = AttributeInfo
  {
    slot : 0,
    buffer : position_buffer,
    descriptor : BufferDescriptor::new::< [ f32; 2 ] >(),
    bounding_box : Default::default(),
  };

  let tex_coord_buffer = gl::buffer::create( &gl )?;
  gl::buffer::upload( &gl, &tex_coord_buffer, tex_coords.as_slice(), GL::STATIC_DRAW );
  let tex_coord_info = AttributeInfo
  {
    slot : 2,
    buffer : tex_coord_buffer,
    descriptor : BufferDescriptor::new::< [ f32; 2 ] >(),
    bounding_box : Default::default(),
  };

  let mut geometry = Geometry::new( &gl )?;
  geometry.vertex_count = positions.len() as u32;
  geometry.add_attribute( &gl, "position", pos_info, false )?;
  geometry.add_attribute( &gl, "tex_coord", tex_coord_info, false )?;
  gl.bind_vertex_array( None );

  Ok( geometry )
}

fn create_line_geometry( gl : &GL ) -> Result< Geometry, minwebgl::WebglError >
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
  geometry.vertex_count = positions.len() as u32;
  geometry.add_attribute( &gl, "position", pos_info, false )?;
  gl.bind_vertex_array( None );

  Ok( geometry )
}

fn tex_coords( positions : &[ f32 ] ) -> Vec< f32 >
{
  let BoundingBox { min, max } = BoundingBox::compute2d( &positions );
  let Vector( [ x_min, y_min, .. ] ) = min;
  let Vector( [ x_max, y_max, .. ] ) = max;

  let dist_x = x_max - x_min;
  let dist_y = y_max - y_min;

  let mut tex_coords = vec![];
  for pos in positions.chunks_exact( 2 )
  {
    let x = ( pos[ 0 ] * 0.982 - x_min ) / dist_x;
    let y = ( pos[ 1 ] * 0.982 - y_min ) / dist_y;
    let y = 1.0 - y;
    tex_coords.push( x );
    tex_coords.push( y );
  }

  tex_coords
}

fn create_shader( gl : &GL ) -> Result< gl::shader::Program, minwebgl::WebglError >
{
  let vert = include_str!( "../shaders/main.vert" );
  let frag = include_str!( "../shaders/main.frag" );
  gl::shader::Program::new( gl.clone(), vert, frag )
}

fn create_line_shader( gl : &GL ) -> Result< gl::shader::Program, minwebgl::WebglError >
{
  let vert = include_str!( "../shaders/main.vert" );
  let frag = include_str!( "../shaders/line.frag" );
  gl::shader::Program::new( gl.clone(), vert, frag )
}

fn load_sprite_sheet
(
  gl : &GL,
  document : &web_sys::Document,
  src : &str
) -> Option< WebGlTexture >
{
  let img = document.create_element( "img" )
  .unwrap()
  .dyn_into::< HtmlImageElement >()
  .unwrap();
  img.style().set_property( "display", "none" ).unwrap();
  let texture = gl.create_texture();

  // TODO: use size of image
  let on_load : Closure< dyn Fn() > = Closure::new
  ({
    let gl = gl.clone();
    let img = img.clone();
    let texture = texture.clone();
    move ||
    {
      gl::texture::d2::upload_no_flip( &gl, texture.as_ref(), &img );
      gl::texture::d2::filter_nearest( &gl );
      img.remove();
    }
  });
  img.set_onload( Some( on_load.as_ref().unchecked_ref() ) );
  img.set_src( &src );
  on_load.forget();

  texture
}

fn setup_select_element( document : &web_sys::Document ) -> HtmlSelectElement
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

fn get_variant( select_element : &HtmlSelectElement ) -> TileValue
{
  let value = select_element.value();
  let variant = TileValue::from_str( &value ).unwrap();
  variant
}

fn setup_download_button
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

fn setup_drop_zone
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

use minwebgl as gl;
use serde::Deserialize;
use std::collections::HashMap;
use tiles_tools::{ coordinates::{ hexagonal, pixel::Pixel }, geometry };
use hexagonal::Coordinate;
use gl::{ JsCast as _, F32x2, I32x2, Vector, GL, BufferDescriptor };
use web_sys::{ wasm_bindgen::prelude::*, HtmlCanvasElement, HtmlImageElement, WebGlTexture };
use browser_input::{ keyboard::KeyboardKey, mouse::MouseButton, Action, Event, EventType };
use renderer::webgl::{ Geometry, AttributeInfo };

type Axial = Coordinate< hexagonal::Axial, hexagonal::Pointy >;

enum TileValue
{
  Empty
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
  gl.viewport( 0, 0, width, height );

  let hexagon = create_hexagon_geometry( &gl )?;
  hexagon.bind( &gl );
  let shader = create_shader( &gl )?;
  shader.activate();

  let sprite_sheet_path = "static/kenney_hexagon_pack/Spritesheets/hexagonAll_sheet.png";
  let sprite_sheet = load_sprite_sheet( &gl, &document, sprite_sheet_path );
  gl.bind_texture( GL::TEXTURE_2D, sprite_sheet.as_ref() );

  let atlas_size = 2048.0f32;
  let atlas = gl::file::load( "kenney_hexagon_pack/Spritesheets/hexagonAll_sheet.xml" ).await.unwrap();
  let atlas = String::from_utf8( atlas ).unwrap();
  let atlas : TextureAtlas = quick_xml::de::from_str( &atlas ).unwrap();
  let sprite = atlas.sub_textures.iter().find( | item | item.name == "medieval_church.png" ).unwrap();
  let sprite_offset = F32x2::from_array( [ sprite.x as f32, sprite.y as f32 ] ) / atlas_size;
  let sprite_size = F32x2::from_array( [ sprite.width as f32, sprite.height as f32 ] ) / atlas_size;

  let mut map = HashMap::< Axial, TileValue >::new();
  let mut input = browser_input::Input::new( Some( canvas.dyn_into().unwrap() ) );
  let dpr = dpr as f32;
  let zoom = 0.1;
  let inv_canvas_size = F32x2::new( 1.0 / width as f32, 1.0 / height as f32 );
  let ascpect_scale = F32x2::new( 1.0, width as f32 / height as f32 );
  let mut camera_pos = F32x2::default();
  let mut last_pointer_pos : Option< I32x2 > = None;
  let move_speed = 15.0;
  let canvas_res = F32x2::new( width as f32, height as f32 );

  let update = move | _ |
  {
    input.update_state();

    if input.is_key_down( KeyboardKey::Space )
    && input.is_button_down( MouseButton::Main )
    {
      if let Some( last_pointer_pos ) = last_pointer_pos
      {
        let Vector ( [ pos_x, pos_y ] ) = input.pointer_position();
        let Vector ( [ last_pos_x, last_pos_y ] ) = last_pointer_pos;
        let mut movement = ( F32x2::new( pos_x as f32, pos_y as f32 )
        - F32x2::new( last_pos_x as f32, last_pos_y as f32 ) ) / canvas_res;
        movement[ 1 ] = -movement[ 1 ];
        camera_pos += movement * move_speed;
      }
    }
    else if input.is_button_down( MouseButton::Main )
    {
      let Vector ( [ x, y ] ) = input.pointer_position();

      let position = F32x2::new( x as f32 * dpr, y as f32 * dpr );
      let position = ( position * inv_canvas_size - 0.5 ) * 2.0;
      let position : Pixel = ( position / ( zoom * ascpect_scale ) - camera_pos ).into();
      let coordinate = position.into();
      gl::info!( "{coordinate:?}" );
      map.insert( coordinate, TileValue::Empty );
    }
    last_pointer_pos = Some( input.pointer_position() );
    input.clear_events();

    for coord in map.keys()
    {
      let axial : Axial = ( *coord ).into();
      let mut position : Pixel = axial.into();
      position.data[ 1 ] = -position.data[ 1 ];
      gl.vertex_attrib2fv_with_f32_array( 1, &position.data );
      gl.vertex_attrib2fv_with_f32_array( 3, sprite_offset.as_slice() );
      gl.vertex_attrib2fv_with_f32_array( 4, sprite_size.as_slice() );
      shader.uniform_upload( "u_scale", ( zoom * ascpect_scale ).as_slice() );
      shader.uniform_upload( "u_camera_pos", camera_pos.as_slice() );
      hexagon.draw( &gl );
    }

    true
  };
  gl::exec_loop::run( update );

  Ok( () )
}

fn create_hexagon_geometry( gl : &GL ) -> Result< Geometry, minwebgl::WebglError >
{
  let positions = geometry::hexagon_triangles_with_tranform
  (
    gl::math::mat2x2h::rot( 30.0f32.to_radians() )
  );
  let tex_coords = tex_coords( &positions );
  // gl::info!( "{tex_coords:?}" );

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

  Ok( geometry )
}

fn tex_coords( positions : &[ f32 ] ) -> Vec< f32 >
{
  let mut positions = positions.to_vec();

  let mut x_min = f32::MAX;
  let mut x_max = f32::MIN;
  let mut y_min = f32::MAX;
  let mut y_max = f32::MIN;

  for pos in positions.chunks_exact_mut( 2 )
  {
    x_min = x_min.min( pos[ 0 ] );
    x_max = x_max.max( pos[ 0 ] );
    y_min = y_min.min( pos[ 1 ] );
    y_max = y_max.max( pos[ 1 ] );
    // make hexagon a little smaller to remove transparent edges from tile sheet
    pos[ 0 ] *= 0.982;
    pos[ 1 ] *= 0.982;
  }

  let dist_x = x_max - x_min;
  let dist_y = y_max - y_min;

  // gl::info!( "x_min: {x_min}" );
  // gl::info!( "x_max: {x_max}" );
  // gl::info!( "y_min: {y_min}" );
  // gl::info!( "y_max: {y_max}" );

  // gl::info!( "dist_x: {dist_x}" );
  // gl::info!( "dist_y: {dist_y}" );

  let mut tex_coords = vec![];
  for pos in positions.chunks_exact( 2 )
  {
    let x = ( pos[ 0 ] - x_min ) / dist_x;
    let y = ( pos[ 1 ] - y_min ) / dist_y;
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

fn load_sprite_sheet( gl : &GL, document : &web_sys::Document, src : &str ) -> Option< WebGlTexture >
{
  let img = document.create_element( "img" )
  .unwrap()
  .dyn_into::< HtmlImageElement >()
  .unwrap();
  img.style().set_property( "display", "none" ).unwrap();

  let texture = gl.create_texture();

  let on_load : Closure< dyn Fn() > = Closure::new
  ({
    let gl = gl.clone();
    let img = img.clone();
    let src = src.to_owned();
    let texture = texture.clone();
    move ||
    {
      gl.bind_texture( gl::TEXTURE_2D, texture.as_ref() );
      // gl.pixel_storei( gl::UNPACK_FLIP_Y_WEBGL, 1 );
      gl.tex_image_2d_with_u32_and_u32_and_html_image_element
      (
        gl::TEXTURE_2D,
        0,
        gl::RGBA as i32,
        gl::RGBA,
        gl::UNSIGNED_BYTE,
        &img
      ).expect( "Failed to upload data to texture" );
      // gl.pixel_storei( gl::UNPACK_FLIP_Y_WEBGL, 0 );

      gl::texture::d2::filter_nearest( &gl );
      gl::texture::d2::wrap_clamp( &gl );
      web_sys::Url::revoke_object_url( &src ).unwrap();
      img.remove();
    }
  });

  img.set_onload( Some( on_load.as_ref().unchecked_ref() ) );
  img.set_src( &src );
  on_load.forget();

  texture
}

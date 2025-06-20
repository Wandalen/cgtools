pub mod helper;
pub mod triaxial;

use helper::*;
use triaxial::*;
use minwebgl as gl;
use browser_input::{ keyboard::KeyboardKey, mouse::MouseButton, Event, EventType };
use std::{ cell::RefCell, collections::{ HashMap, HashSet }, rc::Rc };
use tiles_tools::coordinates::{ hexagonal, pixel::Pixel };
use renderer::webgl::{ AttributeInfo, Geometry };
use strum::{ AsRefStr, EnumIter, EnumString };
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
};
use web_sys::
{
  HtmlCanvasElement,
  HtmlSelectElement,
  WebGlTexture,
};

pub type Axial = Coordinate< hexagonal::Axial, hexagonal::Flat >;

#[ derive( Debug, Serialize, Deserialize, Clone, Copy ) ]
pub struct Tile
{
  pub value : TileValue,
  // TODO: use new-type
  pub owner : u8,
}

#[ derive( Debug, Clone, Copy, AsRefStr, EnumIter, EnumString, Serialize, Deserialize ) ]
pub enum TileValue
{
  Empty,
  Capital,
  Castle,
  Pine,
  Palm,
  Peasant,
  Spearman,
  Knight,
  Baron,
}

#[ derive( Debug, Clone, Copy, AsRefStr, EnumIter, EnumString ) ]
enum EditMode
{
  EditTiles,
  EditRivers,
}

#[ derive( Debug, Default ) ]
pub struct Map
{
  tile_map : HashMap< Axial, Tile >,
  river_map : HashSet< TriAxial >,
}

impl Map
{
  pub fn to_json( &self ) -> String
  {
    let tile_map = self.tile_map.iter().map( | ( k, v ) | ( *k, *v ) ).collect();
    let river_map = self.river_map.iter().copied().collect();
    let serde = MapSerde{ tile_map, river_map };
    serde_json::to_string( &serde ).unwrap()
  }

  pub fn from_json( json : &str ) -> Self
  {
    let MapSerde { tile_map, river_map } : MapSerde = serde_json::from_str( json ).unwrap();
    Self
    {
      tile_map : HashMap::from_iter( tile_map.into_iter() ),
      river_map : HashSet::from_iter( river_map.into_iter() )
    }
  }
}

#[ derive( Debug, Serialize, Deserialize ) ]
struct MapSerde
{
  tile_map : Vec< ( Axial, Tile ) >,
  river_map : Vec< TriAxial >,
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
  gl.enable( GL::BLEND );
  gl.blend_func( GL::SRC_ALPHA, GL::ONE_MINUS_SRC_ALPHA );

  let hexagon = create_hexagon_geometry( &gl )?;
  let outline = create_line_geometry( &gl )?;

  let hexagon_shader = haxagon_shader( &gl )?;
  let outline_shader = line_shader( &gl )?;
  let river_shader = river_shader( &gl )?;
  let sprite_shader = sprite_shader( &gl )?;

  let sprites = load_sprites( &gl, &document );

  let tile_select_element = setup_select::< TileValue >( &document, "tile" );
  let edit_mode_select_element = setup_select::< EditMode >( &document, "edit-mode" );
  let player_select_element = document.get_element_by_id( "player" )
  .unwrap()
  .dyn_into::< HtmlSelectElement >()
  .unwrap();

  let map = Rc::new( RefCell::new( Map::default() ) );
  setup_download_button( &document, map.clone() );
  setup_drop_zone( &document, map.clone() );

  let mut input = browser_input::Input::new
  (
    Some( canvas.dyn_into().unwrap() ),
    move | e |
    {
      let coord = gl::F64x2::new( e.client_x() as f64, e.client_y() as f64 ) * dpr;
      I32x2::from_array( [ coord.x() as i32, coord.y() as i32 ] )
    },
  );

  let mut zoom = 1.0;
  let zoom_factor = 0.75;
  let inv_canvas_size = F32x2::new( 1.0 / width as f32, 1.0 / height as f32 );
  let aspect = width as f32 / height as f32;
  let aspect = F32x2::new( 1.0 / aspect, 1.0 );
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

    let pointer_pos = screen_to_world( pointer_pos, inv_canvas_size, aspect, zoom );
    let mut pixel = pointer_pos - camera_pos;
    pixel[ 1 ] = -pixel[ 1 ];
    let pixel : Pixel = pixel.into();
    let hexagon_coordinate : Axial = pixel.into();

    // Camera pan
    if input.is_key_down( KeyboardKey::Space )
    && input.is_button_down( MouseButton::Main )
    {
      if let Some( last_pointer_pos ) = last_pointer_pos
      {
        let last_pointer_pos = screen_to_world
        (
          last_pointer_pos,
          inv_canvas_size,
          aspect,
          zoom
        );
        let movement = pointer_pos - last_pointer_pos;
        camera_pos += movement;
      }
    }
    // Map editing
    else
    {
      let mode = get_variant::< EditMode >( &edit_mode_select_element );
      let main_button = input.is_button_down( MouseButton::Main );
      let secondary_button = input.is_button_down( MouseButton::Secondary );

      match mode
      {
        EditMode::EditTiles =>
        {
          // Adding tile
          if main_button
          {
            let variant = get_variant( &tile_select_element );
            let owner : u8 = player_select_element.value().parse().unwrap();
            map.borrow_mut().tile_map.insert( hexagon_coordinate, Tile { value : variant, owner } );
          }
          // Removing tile
          else if secondary_button
          {
            map.borrow_mut().tile_map.remove( &hexagon_coordinate );
          }
        },
        EditMode::EditRivers =>
        {
          // Adding tile
          if main_button
          {
            map.borrow_mut().river_map.insert( TriAxial::from_point( pixel.x(), pixel.y() ) );
          }
          // Removing tile
          else if secondary_button
          {
            map.borrow_mut().river_map.remove( &TriAxial::from_point( pixel.x(), pixel.y() ) );
          }
        },
      }
    }

    last_pointer_pos = Some( input.pointer_position() );

    input.clear_events();

    gl.clear( GL::COLOR_BUFFER_BIT );

    hexagon_shader.activate();
    hexagon_shader.uniform_upload( "u_scale", ( zoom * aspect ).as_slice() );
    hexagon_shader.uniform_upload( "u_camera_pos", camera_pos.as_slice() );

    outline_shader.activate();
    outline_shader.uniform_upload( "u_scale", ( zoom * aspect ).as_slice() );
    outline_shader.uniform_upload( "u_camera_pos", camera_pos.as_slice() );

    sprite_shader.activate();
    sprite_shader.uniform_upload( "u_scale", ( zoom * aspect ).as_slice() );
    sprite_shader.uniform_upload( "u_camera_pos", camera_pos.as_slice() );

    river_shader.activate();
    river_shader.uniform_upload( "u_scale", ( zoom * aspect ).as_slice() );
    river_shader.uniform_upload( "u_camera_pos", camera_pos.as_slice() );

    // Order tiles
    let mut tiles : Vec< _ > = map.borrow().tile_map.iter().map( | ( &k, &v ) | ( k, v ) ).collect();
    tiles.sort_by_key( | v | ( v.0.r, v.0.q ) );

    for ( coord, Tile { value, owner } ) in &tiles
    {
      let mut position : Pixel = ( *coord ).into();
      position.data[ 1 ] = -position.data[ 1 ];

      hexagon.bind( &gl );
      hexagon_shader.activate();
      gl.vertex_attrib2fv_with_f32_array( 1, &position.data );
      gl.vertex_attrib_i4i( 5, *owner as i32, 0, 0, 0 );
      hexagon.draw( &gl );

      outline.bind( &gl );
      outline_shader.activate();
      gl.vertex_attrib2fv_with_f32_array( 1, &position.data );
      outline.draw( &gl );

      gl.bind_vertex_array( None );

      let sprite_index = *value as usize;
      if sprite_index == 0 { continue; }
      let ( sprite, size ) = &sprites[ sprite_index - 1 ];
      let sprite_size = [ size.borrow()[ 0 ] as f32 * 0.5, size.borrow()[ 1 ] as f32 * 0.5 ];
      sprite_shader.activate();
      gl.vertex_attrib2fv_with_f32_array( 0, &position.data );
      gl.vertex_attrib2fv_with_f32_array( 1, sprite_size.as_slice() );
      gl.vertex_attrib1f( 2, 0.015 );
      gl.bind_texture( GL::TEXTURE_2D, sprite.as_ref() );
      gl.draw_arrays( GL::TRIANGLE_STRIP, 0, 4 );
    }

    // Draw rivers
    let pairs = find_pairs( &map.borrow().river_map );
    river_shader.activate();
    for [ tri1, tri2 ] in pairs
    {
      let mut p1 : F32x2 = tri1.to_point().into();
      p1[ 1 ] = -p1[ 1 ];
      let mut p2 : F32x2 = tri2.to_point().into();
      p2[ 1 ] = -p2[ 1 ];
      let center = ( p1 + p2 ) / 2.0;
      let lenght = p1.distance( &p2 ) / 2.0;
      let dx = p2.x() - p1.x();
      let dy = p2.y() - p1.y();
      let angle = dy.atan2( dx );
      let rot = gl::math::d2::mat2x2h::rot( angle );
      let translate = gl::math::d2::mat2x2h::translate( center );
      let scale = gl::math::d2::mat2x2h::scale( [ lenght, 0.1 ] );
      let transform = translate * rot * scale;

      river_shader.uniform_matrix_upload( "u_transform", transform.raw_slice(), true );
      gl.draw_arrays( GL::TRIANGLE_STRIP, 0, 4 );
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
  aspect : F32x2,
  zoom : f32
) -> F32x2
{
  let Vector ( [ x, y ] ) = screen;
  let screenf32 = F32x2::new( x as f32, y as f32 );
  let mut world = ( screenf32 * inv_canvas_size - 0.5 ) * 2.0 / ( zoom * aspect );
  world[ 1 ] = -world[ 1 ];
  world
}

fn find_pairs( map : &HashSet< TriAxial > ) -> HashSet< [ TriAxial; 2 ] >
{
  let mut pairs = HashSet::< [ TriAxial; 2 ] >::new();

  for tri in map
  {
    let neighbors = tri.neighbors();
    for neighbor in neighbors
    {
      if map.contains( &neighbor )
      {
        let mut pair = [ *tri, neighbor ];
        // sort to exclude duplication
        pair.sort_by_key( | item | ( item.a, item.b, item.c ) );
        pairs.insert( pair );
      }
    }
  }

  pairs
}

fn create_hexagon_geometry( gl : &GL ) -> Result< Geometry, minwebgl::WebglError >
{
  let positions = tiles_tools::geometry::hexagon_triangles();
  // let tex_coords = tex_coords( &positions );

  let position_buffer = gl::buffer::create( &gl )?;
  gl::buffer::upload( &gl, &position_buffer, positions.as_slice(), GL::STATIC_DRAW );
  let pos_info = AttributeInfo
  {
    slot : 0,
    buffer : position_buffer,
    descriptor : BufferDescriptor::new::< [ f32; 2 ] >(),
    bounding_box : Default::default(),
  };

  // let tex_coord_buffer = gl::buffer::create( &gl )?;
  // gl::buffer::upload( &gl, &tex_coord_buffer, tex_coords.as_slice(), GL::STATIC_DRAW );
  // let tex_coord_info = AttributeInfo
  // {
  //   slot : 2,
  //   buffer : tex_coord_buffer,
  //   descriptor : BufferDescriptor::new::< [ f32; 2 ] >(),
  //   bounding_box : Default::default(),
  // };

  let mut geometry = Geometry::new( &gl )?;
  geometry.vertex_count = positions.len() as u32;
  geometry.add_attribute( &gl, "position", pos_info, false )?;
  // geometry.add_attribute( &gl, "tex_coord", tex_coord_info, false )?;
  gl.bind_vertex_array( None );

  Ok( geometry )
}

/*
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
*/

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

fn haxagon_shader( gl : &GL ) -> Result< gl::shader::Program, minwebgl::WebglError >
{
  let vert = include_str!( "../shaders/main.vert" );
  let frag = include_str!( "../shaders/main.frag" );
  gl::shader::Program::new( gl.clone(), vert, frag )
}

fn line_shader( gl : &GL ) -> Result< gl::shader::Program, minwebgl::WebglError >
{
  let vert = include_str!( "../shaders/main.vert" );
  let frag = include_str!( "../shaders/line.frag" );
  gl::shader::Program::new( gl.clone(), vert, frag )
}

fn river_shader( gl : &GL ) -> Result< gl::shader::Program, minwebgl::WebglError >
{
  let vert = include_str!( "../shaders/river.vert" );
  let frag = include_str!( "../shaders/river.frag" );
  gl::shader::Program::new( gl.clone(), vert, frag )
}

fn sprite_shader( gl : &GL ) -> Result< gl::shader::Program, minwebgl::WebglError >
{
  let vert = include_str!( "../shaders/sprite.vert" );
  let frag = include_str!( "../shaders/sprite.frag" );
  gl::shader::Program::new( gl.clone(), vert, frag )
}

fn load_sprites( gl : &GL, document : &web_sys::Document )
-> Vec< ( Option< WebGlTexture >, Rc< RefCell< gl::U32x2> > ) >
{
  [
    load_texture
    (
      &gl, &document, "static/kenney_hexagon_pack/house.png"
    ),
    load_texture
    (
      &gl, &document, "static/kenney_hexagon_pack/castle_small.png"
    ),
    load_texture
    (
      &gl, &document, "static/kenney_hexagon_pack/treeRound_large.png"
    ),
    load_texture
    (
      &gl, &document, "static/kenney_hexagon_pack/cactus1.png"
    )
  ].into()
}

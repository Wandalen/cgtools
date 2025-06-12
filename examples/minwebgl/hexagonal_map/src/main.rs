mod blob;
mod sprite;
mod helper;
mod tri;

use helper::*;
use minwebgl as gl;
use browser_input::{ keyboard::KeyboardKey, mouse::MouseButton, Event, EventType };
use std::{ cell::RefCell, collections::HashMap, rc::Rc };
use tiles_tools::coordinates::{ hexagonal, pixel::Pixel };
use renderer::webgl::{ AttributeInfo, Geometry };
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
  HtmlCanvasElement,
  HtmlImageElement,
  HtmlSelectElement,
  WebGlTexture,
};

type Axial = Coordinate< hexagonal::Axial, hexagonal::Flat >;

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

  hexagon.bind( &gl );
  let hexagon_shader = create_shader( &gl )?;
  let line_shader = create_line_shader( &gl )?;
  let sprite_shader = sprite::sprite_shader( &gl )?;

  // let size = Default::default();
  // let sprite = load_texture( &gl, &document, "static/kenney_hexagon_pack/castle_small.png", size ).unwrap();
  // let sprite_size = [ 106.0f32 * 0.5, 94.0 * 0.5 ];
  // let sprite = sprite::Sprite::new( [ 106, 94, ].into(), sprite );

  let sprites = load_sprites( &gl, &document );
  // let atlas_size = 2048.0f32;
  // let atlas = gl::file::load
  // (
  //   "kenney_hexagon_pack/Spritesheets/hexagonAll_sheet.xml"
  // ).await.unwrap();
  // let atlas = String::from_utf8( atlas ).unwrap();
  // let atlas : TextureAtlas = quick_xml::de::from_str( &atlas ).unwrap();

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

    hexagon_shader.activate();
    hexagon_shader.uniform_upload( "u_scale", ( zoom * aspect ).as_slice() );
    hexagon_shader.uniform_upload( "u_camera_pos", camera_pos.as_slice() );
    gl.vertex_attrib1f( 6, 1.0 );
    line_shader.activate();
    line_shader.uniform_upload( "u_scale", ( zoom * aspect ).as_slice() );
    line_shader.uniform_upload( "u_camera_pos", camera_pos.as_slice() );
    sprite_shader.activate();
    sprite_shader.uniform_upload( "u_scale", ( zoom * aspect ).as_slice() );
    sprite_shader.uniform_upload( "u_camera_pos", camera_pos.as_slice() );

    // let g = tri::TriangleGrid::new( 1.0, tri::HexOrientation::Flat );
    // let dual = g.hex_to_triangle_dual( map.borrow().keys().map( | Coordinate { q, r, .. } | tri::HexAxial::new( *q, *r ) ) );

    let mut tiles : Vec< _ > = map.borrow().iter().map( | ( &k, &v ) | ( k, v ) ).collect();
    tiles.sort_by_key( | v | ( v.0.r, v.0.q ) );

    for ( coord, Tile { value, owner } ) in &tiles
    {
      let axial : Axial = ( *coord ).into();

      // let sprite = value.to_asset( &atlas );

      // let sprite_offset = F32x2::from_array( [ sprite.x as f32, sprite.y as f32 ] ) / atlas_size;
      // let sprite_size = F32x2::from_array( [ sprite.width as f32, sprite.height as f32 ] ) / atlas_size;
      let mut position : Pixel = axial.into();
      position.data[ 1 ] = -position.data[ 1 ];

      hexagon.bind( &gl );
      hexagon_shader.activate();
      gl.vertex_attrib2fv_with_f32_array( 1, &position.data );
      // gl.vertex_attrib2fv_with_f32_array( 3, sprite_offset.as_slice() );
      // gl.vertex_attrib2fv_with_f32_array( 4, sprite_size.as_slice() );
      gl.vertex_attrib1f( 5, *owner as f32 );
      hexagon.draw( &gl );

      outline.bind( &gl );
      line_shader.activate();
      gl.vertex_attrib2fv_with_f32_array( 1, &position.data );
      outline.draw( &gl );

      gl.bind_vertex_array( None );

      sprite_shader.activate();
      let sprite_index = *value as usize;
      if sprite_index == 0 { continue; }
      let ( sprite, size ) = &sprites[ sprite_index - 1 ];
      gl.bind_texture( GL::TEXTURE_2D, sprite.as_ref() );
      let sprite_size = [ size.borrow()[ 0 ] as f32 * 0.5, size.borrow()[ 1 ] as f32 * 0.5 ];
      gl.vertex_attrib2fv_with_f32_array( 0, &position.data );
      gl.vertex_attrib2fv_with_f32_array( 1, sprite_size.as_slice() );
      // let width = 2.0f32;
      // let height = 3.0f32.sqrt();
      gl.vertex_attrib1f( 2, 0.015 );
      gl.draw_arrays( GL::TRIANGLE_STRIP, 0, 4 );
    }

    let mut dual : Vec< _ > = map.borrow().keys().map( | key | hex_to_dual_triangles( *key ) ).flatten().collect();
    dual.dedup();
    for triangle in dual
    {
      let ( x, y ) = tri_to_pixel(triangle, 1.0 );
      hexagon.bind( &gl );
      hexagon_shader.activate();
      gl.vertex_attrib2fv_with_f32_array( 1, &[ x as f32, -y as f32 ] );
      gl.vertex_attrib1f( 5, 5 as f32 );
      gl.vertex_attrib1f( 6, 0.5 );
      hexagon.draw( &gl );
    }
        // let Coordinate { q, r, _marker } = coordinate;
    // let grid = tri::TriangleGrid::new( 1.0, tri::HexOrientation::Flat );
    // let tripos = grid.pixel_to_triangle( tri::Point { x : hexagonal_pos.x() as f64, y : hexagonal_pos.y() as f64 } );
    // let triangle_vert = grid.triangle_vertices(tri)

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

fn load_texture
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
      gl.generate_mipmap( GL::TEXTURE_2D );
      // gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_MIN_FILTER, GL::LINEAR_MIPMAP_LINEAR as i32 );
      // gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_MAG_FILTER, GL::LINEAR_MIPMAP_LINEAR as i32 );
      // gl::texture::d2::filter_nearest( &gl );
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

#[ derive( Clone, Copy, Debug, Hash, PartialEq, Eq ) ]
struct Tri
{
  a : i32,
  b : i32,
  c : i32, // 0 = up, 1 = down
}

fn hex_to_dual_triangles( hex : Axial ) -> [ Tri; 6 ]
{
  const TRI_OFFSETS : [ ( i32, i32, u8 ); 6 ] =
  [
    (1, 0, 0), (1, 1, 0), (0, 1, 0), (0, 1, 1), (0, 0, 1), (1, 0, 1)
  ];

  TRI_OFFSETS.map
  (
    | ( a, b, c ) |
    Tri
    {
      a: hex.q + a,
      b: hex.r + b,
      c: todo!()
    }
  )
}

fn tri_to_pixel( tri : Tri, hex_size : f32 ) -> ( f32, f32 )
{
  let Tri { a: tq, b: tr, c: orientation } = tri;

  // Position based on hex axial logic
  // let base_x = hex_size * 3.0_f32.sqrt() * ( tq as f32 + tr as f32 / 2.0 );
  // let base_y = hex_size * 1.5 * tr as f32;
  let base_x = hex_size * 1.5 * tq as f32;
  let base_y = hex_size * 3.0f32.sqrt() * ( tr as f32 + tq as f32 / 2.0 );

  // Direction of offset (toward hex corner)
  // For pointy-topped hexes, triangle "points" are vertical-ish
  let angle = match orientation
  {
    0 => 0.0f32,
    1 => std::f32::consts::PI,
    _ => panic!("Invalid triangle orientation"),
  };

  let offset_distance = hex_size / 3.0; // small inward offset
  let offset_x = offset_distance * angle.cos();
  let offset_y = offset_distance * angle.sin();

  ( base_x + offset_x, base_y + offset_y )
}

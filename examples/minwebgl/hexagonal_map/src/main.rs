//! This crate provides a simple hexagonal map editor using WebGL and browser input handling.
//! It allows users to edit tiles, rivers, and player colors on a hexagonal grid.
//! The map can be saved and loaded in JSON format.

#![ allow( clippy::implicit_return ) ]
#![ allow( clippy::default_trait_access ) ]
#![ allow( clippy::min_ident_chars ) ]
#![ allow( clippy::std_instead_of_core ) ]
#![ allow( clippy::cast_precision_loss ) ]
#![ allow( clippy::cast_possible_truncation ) ]
#![ allow( clippy::assign_op_pattern ) ]
#![ allow( clippy::semicolon_if_nothing_returned ) ]
#![ allow( clippy::too_many_lines ) ]
#![ allow( clippy::wildcard_imports ) ]
#![ allow( clippy::needless_borrow ) ]
#![ allow( clippy::cast_possible_wrap ) ]
#![ allow( clippy::redundant_field_names ) ]
#![ allow( clippy::useless_format ) ]
#![ allow( clippy::let_unit_value ) ]
#![ allow( clippy::needless_return ) ]
#![ allow( clippy::cast_sign_loss ) ]
#![ allow( clippy::similar_names ) ]
#![ allow( clippy::needless_continue ) ]
#![ allow( clippy::else_if_without_else ) ]
#![ allow( clippy::unreadable_literal ) ]
#![ allow( clippy::explicit_iter_loop ) ]
#![ allow( clippy::uninlined_format_args ) ]
#![ allow( clippy::collapsible_if ) ]
#![ allow( clippy::unused_async ) ]
#![ allow( clippy::needless_borrows_for_generic_args ) ]
#![ allow( clippy::wrong_self_convention ) ]
#![ allow( clippy::neg_multiply ) ]
#![ allow( clippy::cast_lossless ) ]
#![ allow( clippy::needless_pass_by_value ) ]
#![ allow( clippy::excessive_precision ) ]

mod helper;
mod triaxial;
mod core_game;

use minwebgl as gl;
use browser_input::{ keyboard::KeyboardKey, mouse::MouseButton, Action, Event, EventType };
use gl::{ JsCast as _, I32x2, F32x2, Vector };
use tilemap_renderer::
{
  adapters::webgl::WebGlBackend,
  backend::Backend,
  types::*,
  commands,
  assets,
};
use std::{ cell::RefCell, rc::Rc, str::FromStr, path::PathBuf };
use tiles_tools::coordinates::pixel::Pixel;
use web_sys::HtmlCanvasElement;
use rustc_hash::FxHashMap;
use core_game::Coord;
use triaxial::TriAxial;
use helper::*;

fn f32_to_bytes( data : &[ f32 ] ) -> Vec< u8 >
{
  data.iter().flat_map( | f | f.to_le_bytes() ).collect()
}

/// Builds a Transform that maps from local vertex space to screen pixels,
/// matching the old renderer's camera/zoom/aspect behavior.
fn make_transform
(
  world_pos : [ f32; 2 ],
  rotation : f32,
  obj_scale : [ f32; 2 ],
  camera : [ f32; 2 ],
  zoom : f32,
  width : f32,
  height : f32,
) -> Transform
{
  let ( as_x, as_y ) = if width > height
  {
    ( zoom, zoom * width / height )
  }
  else
  {
    ( zoom * height / width, zoom )
  };

  let world_x = world_pos[ 0 ] + camera[ 0 ];
  let world_y = world_pos[ 1 ] + camera[ 1 ];

  let screen_pos_x = ( world_x * as_x + 1.0 ) * width / 2.0;
  let screen_pos_y = ( world_y * as_y + 1.0 ) * height / 2.0;

  let screen_scale_x = obj_scale[ 0 ] * as_x * width / 2.0;
  let screen_scale_y = obj_scale[ 1 ] * as_y * height / 2.0;

  Transform
  {
    position : [ screen_pos_x, screen_pos_y ],
    rotation,
    scale : [ screen_scale_x, screen_scale_y ],
    ..Default::default()
  }
}

fn main()
{
  gl::browser::setup( Default::default() );
  gl::spawn_local( async move { run().await.unwrap() } );
}

async fn run() -> Result< (), gl::WebglError >
{
  let window = web_sys::window().unwrap();
  let document = window.document().unwrap();
  let fwidth = window.inner_width().unwrap().as_f64().unwrap();
  let fheight = window.inner_height().unwrap().as_f64().unwrap();
  let dpr = window.device_pixel_ratio();
  let gl = gl::context::retrieve_or_make().unwrap();

  let canvas = gl.canvas().unwrap().dyn_into::< HtmlCanvasElement >().unwrap();
  let width = ( fwidth * dpr ) as u32;
  let height = ( fheight * dpr ) as u32;
  canvas.set_width( width );
  canvas.set_height( height );
  browser_input::prevent_rightclick( canvas.clone().dyn_into().unwrap() );

  // ---- Backend setup ----

  let config = RenderConfig
  {
    width,
    height,
    ..Default::default()
  };
  let mut backend = WebGlBackend::new( config, gl.clone() )
  .expect( "backend error" );

  // ---- Geometry IDs ----

  let hexagon_id : ResourceId< asset::Geometry > = ResourceId::new( 0 );
  let outline_id : ResourceId< asset::Geometry > = ResourceId::new( 1 );
  let rectangle_id : ResourceId< asset::Geometry > = ResourceId::new( 2 );

  // ---- Geometry data ----

  let hexagon_mesh = tiles_tools::geometry::hexagon_triangles();
  let outline_mesh = tiles_tools::geometry::hexagon_lines();

  let rect_positions : &[ f32 ] = &
  [
    -1.0, -1.0,
     1.0,  1.0,
    -1.0,  1.0,
    -1.0, -1.0,
     1.0, -1.0,
     1.0,  1.0,
  ];
  let rect_uvs : &[ f32 ] = &
  [
    0.0, 0.0,
    1.0, 1.0,
    0.0, 1.0,
    0.0, 0.0,
    1.0, 0.0,
    1.0, 1.0,
  ];

  // ---- Config ----

  let game_config = include_str!( "../config.json" );
  let game_config = serde_json::from_str::< core_game::Config >( game_config ).unwrap();

  // ---- Texture loading ----

  let mut textures = FxHashMap::< String, ResourceId< asset::Image > >::default();
  let mut image_assets = Vec::new();

  for object in &game_config.object_props
  {
    let Some( sprite ) = &object.sprite else { continue; };
    if textures.contains_key( &sprite.source ) { continue; }
    let id = textures.len() as u32;
    let res_id : ResourceId< asset::Image > = ResourceId::new( id );

    textures.insert( sprite.source.clone(), res_id );

    image_assets.push( assets::ImageAsset
    {
      id : res_id,
      source : assets::ImageSource::Path( PathBuf::from( &sprite.source ) ),
      filter : SamplerFilter::Linear,
      mipmap : MipmapMode::Off,
    });
  }

  // ---- Load assets ----

  let all_assets = assets::Assets
  {
    fonts : vec![],
    images : image_assets,
    sprites : vec![],
    geometries : vec!
    [
      assets::GeometryAsset
      {
        id : hexagon_id,
        positions : assets::Source::Bytes( f32_to_bytes( &hexagon_mesh ) ),
        uvs : None,
        indices : None,
        data_type : assets::DataType::F32,
      },
      assets::GeometryAsset
      {
        id : outline_id,
        positions : assets::Source::Bytes( f32_to_bytes( &outline_mesh ) ),
        uvs : None,
        indices : None,
        data_type : assets::DataType::F32,
      },
      assets::GeometryAsset
      {
        id : rectangle_id,
        positions : assets::Source::Bytes( f32_to_bytes( rect_positions ) ),
        uvs : Some( assets::Source::Bytes( f32_to_bytes( rect_uvs ) ) ),
        indices : None,
        data_type : assets::DataType::F32,
      },
    ],
    gradients : vec![],
    patterns : vec![],
    clip_masks : vec![],
    paths : vec![],
  };

  backend.load_assets( &all_assets )
  .expect( "backend error" );

  // ---- Game state ----

  let map = Rc::new( RefCell::new( core_game::Map::default() ) );
  let loaded_map : Rc< RefCell< Option< String > > > = Default::default();

  let mode_select_variants = [ EditMode::EditTiles, EditMode::EditRivers ].map( | v | v.as_ref().to_string() );
  let mode_select = setup_select( &document, "edit-mode", mode_select_variants.iter() );
  let tile_select = setup_select( &document, "tile", game_config.object_props.iter().map( | p | &p.name ) );
  let player_list = game_config.player_colors.iter().enumerate().map( | ( i, _ ) | i.to_string() ).collect::< Vec< _ > >();
  let player_select = setup_select( &document, "player", player_list.iter() );

  setup_download_button( &document, map.clone() );
  setup_drop_zone( &document, loaded_map.clone() );

  let mut input = browser_input::Input::new
  (
    Some( canvas.clone().dyn_into().unwrap() ),
    move | e |
    {
      let coord = gl::F64x2::new( e.client_x(), e.client_y() ) * dpr;
      I32x2::from_array( [ coord.x() as i32, coord.y() as i32 ] )
    },
  ).expect( "Failed to initialize input" );

  let water_color = [ 0.1, 0.2, 0.4 ];

  let mut zoom = 0.1_f32;
  let zoom_factor = 0.75_f32;
  let inv_canvas_size = F32x2::new( 1.0 / width as f32, 1.0 / height as f32 );
  let aspect = if width > height
  {
    F32x2::from_array( [ 1.0, ( width as f32 / height as f32 ) ] )
  }
  else
  {
    F32x2::from_array( [ ( height as f32 / width as f32 ), 1.0 ] )
  };
  let mut camera_pos = F32x2::default();

  let mut last_pointer_pos : Option< I32x2 > = None;
  let mut river_point1_add = None;
  let mut river_point1_remove = None;

  let update = move | _ |
  {
    input.update_state();

    let w = canvas.width() as f32;
    let h = canvas.height() as f32;

    let mut loaded_map = loaded_map.borrow_mut();
    if let Some( map_json ) = loaded_map.as_ref()
    {
      match serde_json::from_str( &map_json )
      {
        Ok( m ) => *map.borrow_mut() = m,
        Err( e ) => gl::warn!( "{e}" ),
      }
      *loaded_map = None;
    }

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

    let pointer_pos = input.pointer_position();
    let pointer_pos = screen_to_world( pointer_pos, inv_canvas_size, aspect, zoom, h );
    let pixel = pointer_pos - camera_pos;
    let pixel : Pixel = pixel.into();
    let hexagon_coord : Coord = pixel.into();
    let tri_point = TriAxial::from_point( pixel.x(), pixel.y() );

    let edit_mode = EditMode::from_str( &mode_select.value() ).unwrap();

    if edit_mode == EditMode::EditRivers
    {
      for Event { event_type, .. } in input.event_queue().iter()
      {
        if let EventType::PointerButton( _, _, MouseButton::Main, Action::Press ) = event_type
        {
          if river_point1_add.is_none()
          {
            river_point1_add = Some( tri_point );
          }
          else
          {
            let river_point1 = river_point1_add.take().unwrap();
            let river_point2 = tri_point;

            if river_point1.neighbors().contains( &river_point2 )
            {
              map.borrow_mut().rivers.insert( [ river_point1, river_point2 ] );
            }
          }
        }
        if let EventType::PointerButton( _, _, MouseButton::Secondary, Action::Press ) = event_type
        {
          if river_point1_remove.is_none()
          {
            river_point1_remove = Some( tri_point );
          }
          else
          {
            let river_point1 = river_point1_remove.take().unwrap();
            let river_point2 = tri_point;
            if river_point1.neighbors().contains( &river_point2 )
            {
              map.borrow_mut().rivers.remove( &[ river_point1, river_point2 ] );
            }
          }
        }
      }
    }

    if input.is_key_down( KeyboardKey::Space ) && input.is_button_down( MouseButton::Main )
    {
      if let Some( last_pointer_pos ) = last_pointer_pos
      {
        let last_pointer_pos = screen_to_world
        (
          last_pointer_pos,
          inv_canvas_size,
          aspect,
          zoom,
          h,
        );
        let movement = pointer_pos - last_pointer_pos;
        camera_pos += movement;
      }
    }
    else if edit_mode == EditMode::EditTiles
    {
      let main_button = input.is_button_down( MouseButton::Main );
      let secondary_button = input.is_button_down( MouseButton::Secondary );
      let selected_value = tile_select.value();
      let object_index = core_game::ObjectIndex
      (
        game_config.object_props.iter().position( | p | p.name == selected_value ).unwrap() as u32
      );
      let owner_index = core_game::PlayerIndex( player_select.value().parse().unwrap() );

      if main_button
      {
        let tile = core_game::Tile
        {
          object_index : Some( object_index ),
          terrain_index : Default::default(),
          owner_index,
          coord : hexagon_coord,
        };
        map.borrow_mut().tiles.insert( hexagon_coord, tile );
      }
      else if secondary_button
      {
        map.borrow_mut().tiles.remove( &hexagon_coord );
      }
    }

    last_pointer_pos = Some( input.pointer_position() );
    input.clear_events();

    // ---- Build render commands ----

    let cam = [ camera_pos[ 0 ], camera_pos[ 1 ] ];

    let mut render_commands = vec!
    [
      commands::RenderCommand::Clear( commands::Clear { color : [ 0.1, 0.2, 0.3, 1.0 ] } ),
    ];

    for hex in map.borrow().tiles.values()
    {
      let position : Pixel = hex.coord.into();

      // Filled hexagon
      let [ r, g, b ] = game_config.player_colors[ hex.owner_index.0 as usize ];
      let color = [ f32::from( r ) / 255.0, f32::from( g ) / 255.0, f32::from( b ) / 255.0, 1.0 ];
      let tr = make_transform( position.data, 0.0, [ 1.0, 1.0 ], cam, zoom, w, h );
      render_commands.push( commands::RenderCommand::Mesh( commands::Mesh
      {
        transform : tr,
        geometry : hexagon_id,
        fill : FillRef::Solid( color ),
        texture : None,
        topology : Topology::TriangleList,
        blend : BlendMode::Normal,
        clip : None,
      }));

      // Outline
      let tr = make_transform( position.data, 0.0, [ 1.0, 1.0 ], cam, zoom, w, h );
      render_commands.push( commands::RenderCommand::Mesh( commands::Mesh
      {
        transform : tr,
        geometry : outline_id,
        fill : FillRef::Solid( [ 0.0, 0.0, 0.0, 1.0 ] ),
        texture : None,
        topology : Topology::LineList,
        blend : BlendMode::Normal,
        clip : None,
      }));

      // Sprite (textured rectangle)
      let Some( object_index ) = hex.object_index else { continue; };
      let object = &game_config.object_props[ object_index.0 as usize ];
      let Some( sprite ) = &object.sprite else { continue; };
      let Some( tex_res_id ) = textures.get( &sprite.source ) else { continue; };

      let scale = sprite.scale;
      let obj_scale = [ scale, scale ];

      let tr = make_transform( position.data, 0.0, obj_scale, cam, zoom, w, h );
      render_commands.push( commands::RenderCommand::Mesh( commands::Mesh
      {
        transform : tr,
        geometry : rectangle_id,
        fill : FillRef::Solid( [ 1.0, 1.0, 1.0, 1.0 ] ),
        texture : Some( *tex_res_id ),
        topology : Topology::TriangleList,
        blend : BlendMode::Normal,
        clip : None,
      }));
    }

    // Rivers
    let river_width = 0.1;
    for [ p1, p2 ] in &map.borrow().rivers
    {
      let p1 : F32x2 = p1.to_point().into();
      let p2 : F32x2 = p2.to_point().into();
      let center = ( p1 + p2 ) / 2.0;
      let length = p1.distance( &p2 ) / 2.0;
      let dx = p2.x() - p1.x();
      let dy = p2.y() - p1.y();
      let angle = dy.atan2( dx );

      let tr = make_transform( center.into(), angle, [ length, river_width ], cam, zoom, w, h );
      render_commands.push( commands::RenderCommand::Mesh( commands::Mesh
      {
        transform : tr,
        geometry : rectangle_id,
        fill : FillRef::Solid( [ water_color[ 0 ], water_color[ 1 ], water_color[ 2 ], 1.0 ] ),
        texture : None,
        topology : Topology::TriangleList,
        blend : BlendMode::Normal,
        clip : None,
      }));
    }

    let _ = backend.submit( &render_commands );

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
  zoom : f32,
  canvas_height : f32,
) -> F32x2
{
  let Vector ( [ x, y ] ) = screen;
  // Flip browser Y (top-down) to renderer Y (bottom-up)
  let screenf32 = F32x2::new( x as f32, canvas_height - y as f32 );
  ( screenf32 * inv_canvas_size - 0.5 ) * 2.0 / ( zoom * aspect )
}

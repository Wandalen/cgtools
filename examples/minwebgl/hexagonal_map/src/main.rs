//! This crate provides a simple hexagonal map editor using WebGL and browser input handling.
//! It allows users to edit tiles, rivers, and player colors on a hexagonal grid.
//! The map can be saved and loaded in JSON format.

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
  types::{ Transform, RenderConfig, ResourceId, asset, SamplerFilter, MipmapMode, FillRef, Topology, BlendMode, WrapMode },
  commands,
  assets,
};
use std::{ cell::RefCell, rc::Rc, str::FromStr, path::PathBuf };
use tiles_tools::coordinates::pixel::Pixel;
use web_sys::{ HtmlCanvasElement, HtmlSelectElement };
use rustc_hash::FxHashMap;
use core_game::Coord;
use triaxial::TriAxial;
use helper::{ EditMode, select_setup, download_button_setup, drop_zone_setup };

/// Static geometry resource handles shared by every render-command builder this frame.
#[ derive( Clone, Copy ) ]
struct GeometryIds
{
  hexagon : ResourceId< asset::Geometry >,
  outline : ResourceId< asset::Geometry >,
  rectangle : ResourceId< asset::Geometry >,
}

/// Per-frame camera/zoom/screen-size parameters shared by every `transform_make` call.
#[ derive( Clone, Copy ) ]
struct ViewParams
{
  cam : [ f32; 2 ],
  zoom : f32,
  w : f32,
  h : f32,
}

/// The mode/tile/player `<select>` controls used to drive tile editing.
struct Ui
{
  mode : HtmlSelectElement,
  tile : HtmlSelectElement,
  player : HtmlSelectElement,
}

fn f32_to_bytes( data : &[ f32 ] ) -> Vec< u8 >
{
  data.iter().flat_map( | f | f.to_le_bytes() ).collect()
}

/// Builds a Transform that maps from local vertex space to screen pixels,
/// matching the old renderer's camera/zoom/aspect behavior.
fn transform_make( world_pos : [ f32; 2 ], rotation : f32, obj_scale : [ f32; 2 ], view : ViewParams ) -> Transform
{
  let ( as_x, as_y ) = if view.w > view.h
  {
    ( view.zoom, view.zoom * view.w / view.h )
  }
  else
  {
    ( view.zoom * view.h / view.w, view.zoom )
  };

  let world_x = world_pos[ 0 ] + view.cam[ 0 ];
  let world_y = world_pos[ 1 ] + view.cam[ 1 ];

  let screen_pos_x = ( world_x * as_x + 1.0 ) * view.w / 2.0;
  let screen_pos_y = ( world_y * as_y + 1.0 ) * view.h / 2.0;

  let screen_scale_x = obj_scale[ 0 ] * as_x * view.w / 2.0;
  let screen_scale_y = obj_scale[ 1 ] * as_y * view.h / 2.0;

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
  gl::browser::setup( gl::browser::Config::default() );
  gl::spawn_local( async move { app_run() } );
}

fn app_run()
{
  let ( document, canvas, gl, width, height, dpr ) = canvas_setup();

  let game_config = include_str!( "../config.json" );
  let game_config = serde_json::from_str::< core_game::Config >( game_config ).unwrap();

  let ( mut backend, geometry, textures ) = scene_assets_load( gl, width, height, &game_config );

  let map = Rc::new( RefCell::new( core_game::Map::default() ) );
  let loaded_map : Rc< RefCell< Option< String > > > = Rc::default();

  let ui = ui_setup( &document, &game_config, &map, &loaded_map );
  let mut input = input_setup( &canvas, dpr );

  let mut zoom = 0.1_f32;
  let ( inv_canvas_size, aspect ) = screen_params_compute( width, height );
  let mut camera_pos = F32x2::default();

  let mut last_pointer_pos : Option< I32x2 > = None;
  let mut river_point1_add = None;
  let mut river_point1_remove = None;

  let update = move | _ |
  {
    input.state_update();

    let w = canvas.width() as f32;
    let h = canvas.height() as f32;

    loaded_map_sync( &loaded_map, &map );
    zoom_wheel_handle( &input, &mut zoom );

    let pointer_pos = input.pointer_position();
    let pointer_pos = screen_to_world( pointer_pos, inv_canvas_size, aspect, zoom, h );
    let pixel = pointer_pos - camera_pos;
    let pixel : Pixel = pixel.into();
    let hexagon_coord : Coord = pixel.into();
    let tri_point = TriAxial::from_point( pixel.x(), pixel.y() );

    let edit_mode = EditMode::from_str( &ui.mode.value() ).unwrap();

    if edit_mode == EditMode::EditRivers
    {
      river_editing_handle( &input, tri_point, &mut river_point1_add, &mut river_point1_remove, &map );
    }

    if input.is_key_down( KeyboardKey::Space ) && input.is_button_down( MouseButton::Main )
    {
      camera_pan( last_pointer_pos, pointer_pos, inv_canvas_size, aspect, zoom, h, &mut camera_pos );
    }
    else if edit_mode == EditMode::EditTiles
    {
      tile_edit( &input, &ui, &game_config, hexagon_coord, &map );
    }
    else
    {
      // Not panning and not in tile-edit mode (e.g. river-edit mode): nothing to do this frame.
    }

    last_pointer_pos = Some( input.pointer_position() );
    input.events_clear();

    // ---- Build render commands ----

    let view = ViewParams { cam : [ camera_pos[ 0 ], camera_pos[ 1 ] ], zoom, w, h };

    let mut render_commands = vec!
    [
      commands::RenderCommand::Clear( commands::Clear { color : [ 0.1, 0.2, 0.3, 1.0 ] } ),
    ];

    tile_render_commands_push( &map, &game_config, &textures, geometry, view, &mut render_commands );
    river_render_commands_push( &map, geometry.rectangle, view, &mut render_commands );

    let _ = backend.submit( &render_commands );

    true
  };
  gl::exec_loop::run( update );
}

/// Creates the canvas at device-pixel-ratio resolution and retrieves its GL context.
fn canvas_setup() -> ( web_sys::Document, HtmlCanvasElement, web_sys::WebGl2RenderingContext, u32, u32, f64 )
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
  browser_input::rightclick_prevent( &canvas.clone().dyn_into().unwrap() );

  ( document, canvas, gl, width, height, dpr )
}

/// Loads the hexagon/outline/rectangle geometries and every referenced sprite texture into
/// the backend, returning it alongside the geometry handles and the source-path → texture-id map.
fn scene_assets_load
(
  gl : web_sys::WebGl2RenderingContext,
  width : u32,
  height : u32,
  game_config : &core_game::Config,
) -> ( WebGlBackend, GeometryIds, FxHashMap< String, ResourceId< asset::Image > > )
{
  let config = RenderConfig
  {
    width,
    height,
    ..Default::default()
  };
  let mut backend = WebGlBackend::new( config, gl )
  .expect( "backend error" );

  let hexagon_id : ResourceId< asset::Geometry > = ResourceId::new( 0 );
  let outline_id : ResourceId< asset::Geometry > = ResourceId::new( 1 );
  let rectangle_id : ResourceId< asset::Geometry > = ResourceId::new( 2 );

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
      wrap : WrapMode::default(),
    });
  }

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

  backend.assets_load( &all_assets )
  .expect( "backend error" );

  ( backend, GeometryIds { hexagon : hexagon_id, outline : outline_id, rectangle : rectangle_id }, textures )
}

/// Wires up the mode/tile/player `<select>` controls plus the download button and drop zone.
fn ui_setup
(
  document : &web_sys::Document,
  game_config : &core_game::Config,
  map : &Rc< RefCell< core_game::Map > >,
  loaded_map : &Rc< RefCell< Option< String > > >,
) -> Ui
{
  let mode_select_variants = [ EditMode::EditTiles, EditMode::EditRivers ].map( | v | v.as_ref().to_string() );
  let mode_select = select_setup( document, "edit-mode", mode_select_variants.iter() );
  let tile_select = select_setup( document, "tile", game_config.object_props.iter().map( | p | &p.name ) );
  let player_list = game_config.player_colors.iter().enumerate().map( | ( i, _ ) | i.to_string() ).collect::< Vec< _ > >();
  let player_select = select_setup( document, "player", player_list.iter() );

  download_button_setup( document, map.clone() );
  drop_zone_setup( document, loaded_map.clone() );

  Ui { mode : mode_select, tile : tile_select, player : player_select }
}

/// Wires up pointer/keyboard input on `canvas`, scaling pointer coordinates by the device pixel ratio.
fn input_setup( canvas : &HtmlCanvasElement, dpr : f64 ) -> browser_input::Input
{
  browser_input::Input::new
  (
    Some( canvas.clone().dyn_into().unwrap() ),
    move | e |
    {
      // Fix(BUG-053): `client_x`/`client_y` return `i32` or `f64` depending on whether
      // `web_sys_unstable_apis` is active (see minwebgl/src/texture/d2.rs); `.into()` targets
      // `f64` correctly in both cases (`i32: Into<f64>` widens, `f64: Into<f64>` is identity).
      #[ allow( clippy::useless_conversion, reason = "client_x()/client_y() return i32 normally but f64 when web_sys_unstable_apis is active (BUG-053); .into() targets f64 in both cases, which is a real widening conversion for i32 and only a same-type identity conversion for f64, so it cannot be dropped without breaking one configuration" ) ]
      let coord = gl::F64x2::new( e.client_x().into(), e.client_y().into() ) * dpr;
      I32x2::from_array( [ coord.x() as i32, coord.y() as i32 ] )
    },
  ).expect( "Failed to initialize input" )
}

/// Computes the reciprocal canvas size and the aspect-ratio correction vector used by `screen_to_world`.
fn screen_params_compute( width : u32, height : u32 ) -> ( F32x2, F32x2 )
{
  let inv_canvas_size = F32x2::new( 1.0 / width as f32, 1.0 / height as f32 );
  let aspect = if width > height
  {
    F32x2::from_array( [ 1.0, ( width as f32 / height as f32 ) ] )
  }
  else
  {
    F32x2::from_array( [ ( height as f32 / width as f32 ), 1.0 ] )
  };
  ( inv_canvas_size, aspect )
}

/// Applies a dropped-in map JSON payload, if one has arrived since the last frame.
fn loaded_map_sync( loaded_map : &Rc< RefCell< Option< String > > >, map : &Rc< RefCell< core_game::Map > > )
{
  let mut loaded_map = loaded_map.borrow_mut();
  if let Some( map_json ) = loaded_map.as_ref()
  {
    match serde_json::from_str( map_json )
    {
      Ok( m ) => *map.borrow_mut() = m,
      Err( e ) => gl::warn!( "{e}" ),
    }
    *loaded_map = None;
  }
}

/// Applies mouse-wheel deltas queued this frame to the zoom level.
fn zoom_wheel_handle( input : &browser_input::Input, zoom : &mut f32 )
{
  const ZOOM_FACTOR : f32 = 0.75;

  for Event { event_type, .. } in input.event_queue().iter()
  {
    if let EventType::Wheel( Vector( [ _, delta, _ ] ) ) = event_type
    {
      if delta.is_sign_negative()
      {
        *zoom /= ZOOM_FACTOR;
      }
      else
      {
        *zoom *= ZOOM_FACTOR;
      }
    }
  }
}

/// Adds or removes river segments based on this frame's queued pointer-button presses.
fn river_editing_handle
(
  input : &browser_input::Input,
  tri_point : TriAxial,
  river_point1_add : &mut Option< TriAxial >,
  river_point1_remove : &mut Option< TriAxial >,
  map : &Rc< RefCell< core_game::Map > >,
)
{
  for Event { event_type, .. } in input.event_queue().iter()
  {
    if let EventType::PointerButton( _, _, MouseButton::Main, Action::Press ) = event_type
    {
      if river_point1_add.is_none()
      {
        *river_point1_add = Some( tri_point );
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
        *river_point1_remove = Some( tri_point );
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

/// Pans the camera by the pointer movement since last frame (call only while dragging).
fn camera_pan
(
  last_pointer_pos : Option< I32x2 >,
  pointer_pos : F32x2,
  inv_canvas_size : F32x2,
  aspect : F32x2,
  zoom : f32,
  h : f32,
  camera_pos : &mut F32x2,
)
{
  let Some( last_pointer_pos ) = last_pointer_pos else { return; };

  let last_pointer_pos = screen_to_world( last_pointer_pos, inv_canvas_size, aspect, zoom, h );
  let movement = pointer_pos - last_pointer_pos;
  *camera_pos += movement;
}

/// Places or removes a tile at `hexagon_coord` based on which mouse button is held
/// (call only while in tile-edit mode).
fn tile_edit
(
  input : &browser_input::Input,
  ui : &Ui,
  game_config : &core_game::Config,
  hexagon_coord : Coord,
  map : &Rc< RefCell< core_game::Map > >,
)
{
  let main_button = input.is_button_down( MouseButton::Main );
  let secondary_button = input.is_button_down( MouseButton::Secondary );
  let selected_value = ui.tile.value();
  let object_index = core_game::ObjectIndex
  (
    game_config.object_props.iter().position( | p | p.name == selected_value ).unwrap() as u32
  );
  let owner_index = core_game::PlayerIndex( ui.player.value().parse().unwrap() );

  if main_button
  {
    let tile = core_game::Tile
    {
      object_index : Some( object_index ),
      terrain_index : core_game::TerraintIndex::default(),
      owner_index,
      coord : hexagon_coord,
    };
    map.borrow_mut().tiles.insert( hexagon_coord, tile );
  }
  else if secondary_button
  {
    map.borrow_mut().tiles.remove( &hexagon_coord );
  }
  else
  {
    // Neither mouse button held: nothing to edit this frame.
  }
}

/// Appends one filled hexagon, one outline, and (if the tile has a sprite) one textured
/// quad per placed tile.
fn tile_render_commands_push
(
  map : &Rc< RefCell< core_game::Map > >,
  game_config : &core_game::Config,
  textures : &FxHashMap< String, ResourceId< asset::Image > >,
  geometry : GeometryIds,
  view : ViewParams,
  render_commands : &mut Vec< commands::RenderCommand >,
)
{
  for hex in map.borrow().tiles.values()
  {
    let position : Pixel = hex.coord.into();

    // Filled hexagon
    let [ r, g, b ] = game_config.player_colors[ hex.owner_index.0 as usize ];
    let color = [ f32::from( r ) / 255.0, f32::from( g ) / 255.0, f32::from( b ) / 255.0, 1.0 ];
    let tr = transform_make( position.data, 0.0, [ 1.0, 1.0 ], view );
    render_commands.push( commands::RenderCommand::Mesh( commands::Mesh
    {
      transform : tr,
      geometry : geometry.hexagon,
      fill : FillRef::Solid( color ),
      texture : None,
      topology : Topology::TriangleList,
      blend : BlendMode::Normal,
      clip : None,
    }));

    // Outline
    let tr = transform_make( position.data, 0.0, [ 1.0, 1.0 ], view );
    render_commands.push( commands::RenderCommand::Mesh( commands::Mesh
    {
      transform : tr,
      geometry : geometry.outline,
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

    let tr = transform_make( position.data, 0.0, obj_scale, view );
    render_commands.push( commands::RenderCommand::Mesh( commands::Mesh
    {
      transform : tr,
      geometry : geometry.rectangle,
      fill : FillRef::Solid( [ 1.0, 1.0, 1.0, 1.0 ] ),
      texture : Some( *tex_res_id ),
      topology : Topology::TriangleList,
      blend : BlendMode::Normal,
      clip : None,
    }));
  }
}

/// Appends one stretched, rotated rectangle per river segment.
fn river_render_commands_push
(
  map : &Rc< RefCell< core_game::Map > >,
  rectangle_id : ResourceId< asset::Geometry >,
  view : ViewParams,
  render_commands : &mut Vec< commands::RenderCommand >,
)
{
  let water_color = [ 0.1, 0.2, 0.4 ];
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

    let tr = transform_make( center.into(), angle, [ length, river_width ], view );
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

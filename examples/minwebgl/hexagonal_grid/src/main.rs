mod pathfind;

use tiles_tools::
{
  coordinates::{ hexagonal::*, pixel::Pixel },
  layout::*,
  geometry
};

use minwebgl as min;
use min::
{
  math::{ F32x2, IntoVector, mat2x2h },
  Program,
  JsCast,
  canvas::HtmlCanvasElement,
  GL,
  // web::log::info,
  // aaa : this import does not work, but not clear why
  // make it working please
  // it just does not work 😕
};
use web_sys::{ wasm_bindgen::prelude::Closure, KeyboardEvent, MouseEvent };
use std::{ cell::RefCell, collections::HashMap, rc::Rc };

fn main() -> Result< (), min::WebglError >
{
  draw_hexes()
}

fn draw_hexes() -> Result< (), minwebgl::WebglError >
{
  min::browser::setup( Default::default() );
  let o = min::context::ContexOptions::new().reduce_dpr( true );
  let context = min::context::retrieve_or_make_with( o )?;
  let canvas = context.canvas().unwrap().dyn_into::< HtmlCanvasElement >().unwrap();
  // used to scale canvas true size to css size
  let dpr = web_sys::window().unwrap().device_pixel_ratio() as f32;
  let canvas_size = ( canvas.width() as f32, canvas.height() as f32 ).into_vector() / dpr;

  // inclusize grid bounds
  let region = [ Coordinate::< Offset< Odd >, Pointy >::new( 0, 0 ), Coordinate::< Offset< _ >, _ >::new( 8, 8 ) ];
  // aaa : why shift_type is not part of layout? o.O
  // aaa : what about type Grid combinging layout and grid size. also grid probably can have offset of orign?
  let rect = RectangularGrid::new( region );

  let grid_center = rect.center();

  min::info!( "grid center: {grid_center:?}" );

  let grid_mesh = geometry::from_iter
  (
    rect.coordinates().map( | c | Into::< Coordinate< Axial, _ > >::into( c ) ),
    || geometry::hexagon_triangles(),
    mat2x2h::rot( 30.0f32.to_radians() ) * mat2x2h::scale( [ 0.9, 0.9 ] )
  );

  let aspect = canvas_size[ 1 ] / canvas_size[ 0 ];
  let scale = 0.07;
  let aspect_scale : F32x2 = [ aspect * scale, scale ].into();
  let scale_m = mat2x2h::scale( aspect_scale.0 );

  let vert = include_str!( "shaders/main.vert" );
  let frag = include_str!( "shaders/main.frag" );
  let hex_shader = Rc::new( Program::new( context.clone(), vert, frag )? );
  hex_shader.activate();

  let grid_geometry = Rc::new
  (
    min::geometry::Positions::new
    (
      context.clone(),
      &grid_mesh, // aaa : iterating all tiles several times is not efficient. is it possible to avoid it?
      2,
    )?
  );
  let outline_geometry = min::geometry::Positions::new
  (
    context.clone(),
    &geometry::hexagon_lines(),
    2,
  )?;
  let hexagon_geometry = Rc::new(min::geometry::Positions::new
  (
    context.clone(),
    &geometry::hexagon_triangles(),
    2,
  )?);


  let translation = mat2x2h::translate( [ -grid_center.x(), grid_center.y() ] );
  let mvp = scale_m * translation;

  context.clear_color( 0.9, 0.9, 0.9, 1.0 );
  context.clear( GL::COLOR_BUFFER_BIT );
  hex_shader.uniform_matrix_upload( "u_mvp", mvp.raw_slice(), true );
  hex_shader.uniform_upload( "u_color", &[ 0.7, 0.7, 0.7, 1.0 ] );
  grid_geometry.activate();
  context.draw_arrays( GL::TRIANGLES, 0, grid_geometry.nvertices );


  ////// GRID DEMO //////


  let mut selected_hex = None;

  let demo1 =
  {
    let context = context.clone();
    let canvas = canvas.clone();
    let grid_geometry = grid_geometry.clone();
    let hex_shader = hex_shader.clone();
    move | e : MouseEvent |
    {
      let rect = canvas.get_bounding_client_rect();
      let canvas_pos = F32x2::new( rect.left() as f32, rect.top() as f32 );
      let half_size : F32x2 = canvas_size / 2.0;
      let cursor_pos = F32x2::new( e.client_x() as f32, e.client_y() as f32 );
      // aaa : where is center? in the middle? what are boundaries -1, +1? explain all that instead of duplicating what is avaliable from code
      // normalize coodinates to NDC [ -1 : 1 ], then apply inverse ascpect scale and offset to grid center
      // this transforms cursor position to the world space
      // then offset it by center of the grid, so that if cursor is in the center of the canvas, it will be in the center of the grid
      let cursor_pos : Pixel = ( ( ( cursor_pos - canvas_pos ) - half_size ) / ( half_size * aspect_scale ) + grid_center ).into(); // aaa : don't use double devission it's confusing and difficult to read. use canonical represenation

      let selected_hex_coord : Coordinate::< Axial, Pointy > = cursor_pos.into();

      if selected_hex.is_some_and( | hex_coord | hex_coord == selected_hex_coord )
      {
        return;
      }
      // aaa : add commented out code to see mouse position in log.
      // min::info!( "selected hex: {selected_hex_coord:?}" );

      selected_hex = Some( selected_hex_coord );

      context.clear( GL::COLOR_BUFFER_BIT );

      // draw grid
      hex_shader.uniform_matrix_upload( "u_mvp", mvp.raw_slice(), true );
      hex_shader.uniform_upload( "u_color", &[ 0.7, 0.7, 0.7, 1.0 ] );
      grid_geometry.activate();
      context.draw_arrays( GL::TRIANGLES, 0, grid_geometry.nvertices );

      let selected_hex_pos : Pixel = selected_hex_coord.into();
      let translation = mat2x2h::translate( [ selected_hex_pos[ 0 ] - grid_center[ 0 ], -selected_hex_pos[ 1 ] + grid_center[ 1 ] ] );
      let mvp = scale_m * translation * mat2x2h::rot( 30.0f32.to_radians() );

      // draw outline
      hex_shader.uniform_matrix_upload( "u_mvp", mvp.raw_slice(), true );
      hex_shader.uniform_upload( "u_color", &[ 0.1, 0.1, 0.1, 1.0 ] );
      outline_geometry.activate();
      context.draw_arrays( GL::LINES, 0, outline_geometry.nvertices ); // aaa : don't use loop geometry, it has limmited suport among backends
                                                                       // i added default lines mesh generation support, but for this webgl rendering i think line loop is okay
                                                                       // aaa : let's use linestrip. rid of loops
    }
  };
  let mouse_move = Closure::< dyn FnMut( _ ) >::new( Box::new( demo1 ) );
  canvas.set_onmousemove( Some( mouse_move.as_ref().unchecked_ref() ) );
  mouse_move.forget();

  let map = Rc::new
  (
    RefCell::new
    (
      HashMap::< Coordinate< Axial, Pointy >, bool >::from_iter( rect.coordinates().map( | c | ( c.into(), true ) ) )
    )
  );
  let demo2 =
  {
    let canvas = canvas.clone();
    let map = map.clone();

    move | e : MouseEvent |
    {
      let rect = canvas.get_bounding_client_rect();
      let canvas_pos = F32x2::new( rect.left() as f32, rect.top() as f32 );
      let half_size : F32x2 = canvas_size / 2.0;
      let cursor_pos = F32x2::new( e.client_x() as f32, e.client_y() as f32 );
      let cursor_pos : Pixel = ( ( ( cursor_pos - canvas_pos ) - half_size ) / ( half_size * aspect_scale ) + grid_center ).into();
      let selected_hex_coord : Coordinate::< Axial, Pointy > = cursor_pos.into();
      let mut map = map.borrow_mut();
      if map.contains_key( &selected_hex_coord )
      {
        map.entry( selected_hex_coord ).and_modify( | v | *v = !*v );
      }
    }
  };
  let mouse_down = Closure::< dyn FnMut( _ ) >::new( Box::new( demo2 ) );
  canvas.set_onmousedown( Some( mouse_down.as_ref().unchecked_ref() ) );
  mouse_down.forget();


  ////// PATHFIND DEMO //////


  let mut selected_hex = None;
  let demo2 =
  {
    let canvas = canvas.clone();
    let context = context.clone();
    let hex_shader = hex_shader.clone();
    let hexagon_geometry = hexagon_geometry.clone();
    let map = map.clone();
    move | e : MouseEvent |
    {
      let rect = canvas.get_bounding_client_rect();
      let canvas_pos = F32x2::new( rect.left() as f32, rect.top() as f32 );
      let half_size : F32x2 = canvas_size / 2.0;
      let cursor_pos = F32x2::new( e.client_x() as f32, e.client_y() as f32 );
      let cursor_pos : Pixel = ( ( ( cursor_pos - canvas_pos ) - half_size ) / ( half_size * aspect_scale ) + grid_center ).into();
      let selected_hex_coord : Coordinate::< Axial, Pointy > = cursor_pos.into();

      if selected_hex.is_some_and( | hex_coord | hex_coord == selected_hex_coord )
      {
        return;
      }

      selected_hex = Some( selected_hex_coord );

      context.clear( GL::COLOR_BUFFER_BIT );

      // draw grid
      hex_shader.uniform_matrix_upload( "u_mvp", mvp.raw_slice(), true );
      hex_shader.uniform_upload( "u_color", &[ 0.7, 0.7, 0.7, 1.0 ] );
      grid_geometry.activate();
      context.draw_arrays( GL::TRIANGLES, 0, grid_geometry.nvertices );

      for ( &coord, _ ) in map.borrow().iter().filter( | ( _, v ) | !**v )
      {
        let hex_pos : Pixel = coord.into();
        let translation = mat2x2h::translate( [ hex_pos[ 0 ] - grid_center[ 0 ], -hex_pos[ 1 ] + grid_center[ 1 ] ] );
        let mvp = scale_m * translation * mat2x2h::rot( 30.0f32.to_radians() ); // * mat2x2h::scale( [ 0.9, 0.9 ] );

        hex_shader.uniform_matrix_upload( "u_mvp", mvp.raw_slice(), true );
        hex_shader.uniform_upload( "u_color", &[ 0.1, 0.1, 0.1, 1.0 ] );
        hexagon_geometry.activate();
        context.draw_arrays( GL::TRIANGLES, 0, hexagon_geometry.nvertices );
      }

      let start = Coordinate::< Axial, _ >::new( 2, 4 );
      let goal = selected_hex_coord;

      let path = pathfind::find_path( &start, &goal, | coord | map.borrow().get( &coord ).copied().unwrap_or_default() );
      if let Some( ( path, _ ) ) = path
      {
        for coord in path
        {
          let hex_pos : Pixel = coord.into();
          let translation = mat2x2h::translate( [ hex_pos[ 0 ] - grid_center[ 0 ], -hex_pos[ 1 ] + grid_center[ 1 ] ] );
          let mvp = scale_m * translation * mat2x2h::rot( 30.0f32.to_radians() ); //* mat2x2h::scale( [ 0.9, 0.9 ] );

          hex_shader.uniform_matrix_upload( "u_mvp", mvp.raw_slice(), true );
          hex_shader.uniform_upload( "u_color", &[ 0.1, 0.6, 0.1, 1.0 ] );
          hexagon_geometry.activate();
          context.draw_arrays( GL::TRIANGLES, 0, hexagon_geometry.nvertices );
        }
      }
    }
  };
  let mouse_move = Closure::< dyn FnMut( _ ) >::new( Box::new( demo2 ) );
  canvas.set_onmousemove( Some( mouse_move.as_ref().unchecked_ref() ) );
  mouse_move.forget();


  let current_color = [ 0.0, 0.0, 0.0 ];
  let mut painting_canvas = tiles_tools::collection::HexArray::< Offset< Odd >, Pointy, [ f32; 3 ] >::new
  (
    [ 41, 41 ].into(),
    [ 20, 20 ].into(),
    || [ 1.0, 1.0, 1.0 ]
  );

  let is_mouse_down = Rc::new( RefCell::new( false ) );

  let mouse_down =
  {
    let is_mouse_down = is_mouse_down.clone();
    move | event : MouseEvent |
    {
      if event.button() == 0 { *( is_mouse_down.borrow_mut() ) = true; }
    }
  };
  let mouse_down = Closure::< dyn FnMut( _ ) >::new( Box::new( mouse_down ) );

  let mouse_up =
  {
    let is_mouse_down = is_mouse_down.clone();
    move | event : MouseEvent |
    {
      if event.button() == 0 { *( is_mouse_down.borrow_mut() ) = false; }
    }
  };
  let mouse_up = Closure::< dyn FnMut( _ ) >::new( Box::new( mouse_up ) );

  canvas.set_onmousedown( Some( mouse_down.as_ref().unchecked_ref() ) );
  canvas.set_onmouseup( Some( mouse_up.as_ref().unchecked_ref() ) );
  mouse_down.forget();
  mouse_up.forget();

  let paint =
  {
    let canvas = canvas.clone();
    let is_mouse_down = is_mouse_down.clone();
    let context = context.clone();
    let hexagon_geometry = hexagon_geometry.clone();
    let hex_shader = hex_shader.clone();
    move | event : MouseEvent |
    {
      if *( is_mouse_down.borrow() )
      {
        let rect = canvas.get_bounding_client_rect();
        let canvas_pos = F32x2::new( rect.left() as f32, rect.top() as f32 );
        let half_size : F32x2 = canvas_size / 2.0;
        let cursor_pos = F32x2::new( event.client_x() as f32, event.client_y() as f32 );
        let cursor_pos : Pixel = ( ( ( cursor_pos - canvas_pos ) - half_size ) / ( half_size * aspect_scale ) ).into();
        let selected_hex_coord : Coordinate::< Axial, Pointy > = cursor_pos.into();

        painting_canvas[ selected_hex_coord ] = current_color;
        // context.clear( GL::COLOR_BUFFER_BIT );
        for ( coord, &[ r, g, b ] ) in painting_canvas.indexed_iter()
        {
          let axial : Coordinate< Axial, _ > = coord.into();
          let hex_pos : Pixel = axial.into();
          let translation = mat2x2h::translate( [ hex_pos[ 0 ], -hex_pos[ 1 ] ] );
          let mvp = scale_m * translation * mat2x2h::rot( 30.0f32.to_radians() );

          hex_shader.uniform_matrix_upload( "u_mvp", mvp.raw_slice(), true );
          hex_shader.uniform_upload( "u_color", &[ r, g, b, 1.0 ] );
          hexagon_geometry.activate();
          context.draw_arrays( GL::TRIANGLES, 0, hexagon_geometry.nvertices );
        }
      }
    }
  };
  let mouse_move = Closure::< dyn FnMut( _ ) >::new( Box::new( paint ) );
  canvas.set_onmousemove( Some( mouse_move.as_ref().unchecked_ref() ) );
  mouse_move.forget();

  Ok( () )
}


#[ derive( Clone, Copy, Debug, PartialEq, Eq ) ]
pub enum Type
{
  Keyboard( KeyboardCode ),
  Mouse( MouseButton ),
  Wheel,
}

#[ derive( Clone, Copy, Debug, PartialEq, Eq ) ]
pub enum Action
{
  Press,
  Release,
}

#[ derive( Clone, Copy, Debug, PartialEq, Eq ) ]
pub struct Event
{
  r#type : Type,
  action : Action,
  alt : bool,
  ctrl : bool,
  shift : bool,
}

pub struct Events
{
  events : Vec< Event >,
  keyboard_state : [ bool; KeyboardCode::COUNT ],
  mouse_state : [ bool; MouseButton::COUNT ],
}

pub struct Input
{
  events : Rc< RefCell< Events > >,
  mouse_closure : Closure< dyn Fn( MouseEvent ) >,
  keyboard_closure : Closure< dyn Fn( KeyboardEvent ) >,
}

impl Input
{
  pub fn new() -> Self
  {
    let events = Events
    {
      events : Vec::new(),
      keyboard_state : [ false; KeyboardCode::COUNT ],
      mouse_state : [ false; MouseButton::COUNT ],
    };

    let input = Rc::new( RefCell::new( events ) );
    let window = web_sys::window().unwrap();
    let mouse_closure =
    {
      let input = input.clone();
      move | event : MouseEvent |
      {
        let button = MouseButton::from_button( event.button() );
        let r#type = Type::Mouse( button );
        let alt = event.alt_key();
        let ctrl = event.ctrl_key();
        let shift = event.shift_key();
        let event = Event
        {
          r#type,
          action : if event.type_() == "mousedown"
          {
            input.borrow_mut().mouse_state[ button as usize ] = true;
            Action::Press
          }
          else
          {
            Action::Release
          },
          alt,
          ctrl,
          shift,
        };
        input.borrow_mut().events.push( event );
      }
    };
    let keyboard_closure =
    {
      let input = input.clone();
      move | event : KeyboardEvent |
      {
        let code = KeyboardCode::from_code( &event.code() );
        let r#type = Type::Keyboard( code );
        let alt = event.alt_key();
        let ctrl = event.ctrl_key();
        let shift = event.shift_key();
        let event = Event
        {
          r#type,
          action : if event.type_() == "keydown"
          {
            input.borrow_mut().keyboard_state[ code as usize ] = true;
            Action::Press
          }
          else
          {
            input.borrow_mut().keyboard_state[ code as usize ] = false;
            Action::Release
          },
          alt,
          ctrl,
          shift,
        };
        input.borrow_mut().events.push( event );
      }
    };

    let mouse_closure = Closure::< dyn Fn( _ ) >::new( Box::new( mouse_closure ) );
    let keyboard_closure = Closure::< dyn Fn( _ ) >::new( Box::new( keyboard_closure ) );

    let p = Input
    {
      events: input,
      mouse_closure,
      keyboard_closure,
    };

    window.add_event_listener_with_callback( "keydown", p.keyboard_closure.as_ref().unchecked_ref() ).unwrap();
    window.add_event_listener_with_callback( "keyup", p.keyboard_closure.as_ref().unchecked_ref() ).unwrap();
    window.add_event_listener_with_callback( "mousedown", p.mouse_closure.as_ref().unchecked_ref() ).unwrap();
    window.add_event_listener_with_callback( "mouseup", p.mouse_closure.as_ref().unchecked_ref() ).unwrap();
    // window.add_event_listener_with_callback( "mousemove", todo!() ).unwrap();
    // window.add_event_listener_with_callback( "wheel", todo!() ).unwrap();

    p
  }

  pub fn poll( &self ) -> Vec< Event >
  {
    let mut events = self.events.borrow_mut();
    let events = std::mem::take( &mut events.events );
    events
  }
}

use std::str::FromStr;
use strum::EnumCount;
/// KeyboardCode represents all possible values for the KeyboardEvent.code property
/// as defined in the UI Events KeyboardEvent code Values specification.
#[ derive( Debug, Clone, Copy, PartialEq, Eq, Hash, EnumCount ) ]
pub enum KeyboardCode {
    // Modifier keys
    AltLeft,
    AltRight,
    ControlLeft,
    ControlRight,
    MetaLeft,
    MetaRight,
    ShiftLeft,
    ShiftRight,

    // Whitespace keys
    Enter,
    Tab,
    Space,

    // Navigation keys
    ArrowDown,
    ArrowLeft,
    ArrowRight,
    ArrowUp,
    End,
    Home,
    PageDown,
    PageUp,

    // UI keys
    Escape,
    CapsLock,
    ScrollLock,
    NumLock,
    PrintScreen,
    Pause,
    ContextMenu,

    // Common function keys
    F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12,
    F13, F14, F15, F16, F17, F18, F19, F20, F21, F22, F23, F24,

    // Editing keys
    Backspace,
    Clear,
    Delete,
    Insert,

    // Alphanumeric keys
    Digit0, Digit1, Digit2, Digit3, Digit4, Digit5, Digit6, Digit7, Digit8, Digit9,
    KeyA, KeyB, KeyC, KeyD, KeyE, KeyF, KeyG, KeyH, KeyI, KeyJ, KeyK, KeyL, KeyM,
    KeyN, KeyO, KeyP, KeyQ, KeyR, KeyS, KeyT, KeyU, KeyV, KeyW, KeyX, KeyY, KeyZ,

    // Numpad keys
    Numpad0, Numpad1, Numpad2, Numpad3, Numpad4, Numpad5, Numpad6, Numpad7, Numpad8, Numpad9,
    NumpadAdd, NumpadSubtract, NumpadMultiply, NumpadDivide, NumpadEnter, NumpadDecimal, NumpadEqual, NumpadComma,

    // Symbol keys
    Backquote,
    BracketLeft,
    BracketRight,
    Comma,
    Period,
    Semicolon,
    Quote,
    Backslash,
    Slash,
    Minus,
    Equal,
    IntlBackslash,
    IntlRo,
    IntlYen,

    // Mobile and special buttons
    AudioVolumeDown,
    AudioVolumeMute,
    AudioVolumeUp,
    BrowserBack,
    BrowserFavorites,
    BrowserForward,
    BrowserHome,
    BrowserRefresh,
    BrowserSearch,
    BrowserStop,
    Eject,
    LaunchApp1,
    LaunchApp2,
    LaunchMail,
    MediaPlayPause,
    MediaStop,
    MediaTrackNext,
    MediaTrackPrevious,
    Power,
    Sleep,
    WakeUp,

    // Extra keys for international keyboards
    Lang1, Lang2, Lang3, Lang4, Lang5,
    Convert,
    NonConvert,
    KanaMode,

    // Unidentified key
    Unidentified,
}

impl KeyboardCode {
    /// Convert a string code value to a KeyboardCode enum variant
    pub fn from_code(code: &str) -> Self {
        KeyboardCode::from_str(code).unwrap_or(KeyboardCode::Unidentified)
    }

    /// Get the string representation of this KeyboardCode
    pub fn as_str(&self) -> &'static str {
        match self {
            // Modifier keys
            KeyboardCode::AltLeft => "AltLeft",
            KeyboardCode::AltRight => "AltRight",
            KeyboardCode::ControlLeft => "ControlLeft",
            KeyboardCode::ControlRight => "ControlRight",
            KeyboardCode::MetaLeft => "MetaLeft",
            KeyboardCode::MetaRight => "MetaRight",
            KeyboardCode::ShiftLeft => "ShiftLeft",
            KeyboardCode::ShiftRight => "ShiftRight",

            // Whitespace keys
            KeyboardCode::Enter => "Enter",
            KeyboardCode::Tab => "Tab",
            KeyboardCode::Space => "Space",

            // Navigation keys
            KeyboardCode::ArrowDown => "ArrowDown",
            KeyboardCode::ArrowLeft => "ArrowLeft",
            KeyboardCode::ArrowRight => "ArrowRight",
            KeyboardCode::ArrowUp => "ArrowUp",
            KeyboardCode::End => "End",
            KeyboardCode::Home => "Home",
            KeyboardCode::PageDown => "PageDown",
            KeyboardCode::PageUp => "PageUp",

            // UI keys
            KeyboardCode::Escape => "Escape",
            KeyboardCode::CapsLock => "CapsLock",
            KeyboardCode::ScrollLock => "ScrollLock",
            KeyboardCode::NumLock => "NumLock",
            KeyboardCode::PrintScreen => "PrintScreen",
            KeyboardCode::Pause => "Pause",
            KeyboardCode::ContextMenu => "ContextMenu",

            // Function keys
            KeyboardCode::F1 => "F1",
            KeyboardCode::F2 => "F2",
            KeyboardCode::F3 => "F3",
            KeyboardCode::F4 => "F4",
            KeyboardCode::F5 => "F5",
            KeyboardCode::F6 => "F6",
            KeyboardCode::F7 => "F7",
            KeyboardCode::F8 => "F8",
            KeyboardCode::F9 => "F9",
            KeyboardCode::F10 => "F10",
            KeyboardCode::F11 => "F11",
            KeyboardCode::F12 => "F12",
            KeyboardCode::F13 => "F13",
            KeyboardCode::F14 => "F14",
            KeyboardCode::F15 => "F15",
            KeyboardCode::F16 => "F16",
            KeyboardCode::F17 => "F17",
            KeyboardCode::F18 => "F18",
            KeyboardCode::F19 => "F19",
            KeyboardCode::F20 => "F20",
            KeyboardCode::F21 => "F21",
            KeyboardCode::F22 => "F22",
            KeyboardCode::F23 => "F23",
            KeyboardCode::F24 => "F24",

            // Editing keys
            KeyboardCode::Backspace => "Backspace",
            KeyboardCode::Clear => "Clear",
            KeyboardCode::Delete => "Delete",
            KeyboardCode::Insert => "Insert",

            // Alphanumeric keys
            KeyboardCode::Digit0 => "Digit0",
            KeyboardCode::Digit1 => "Digit1",
            KeyboardCode::Digit2 => "Digit2",
            KeyboardCode::Digit3 => "Digit3",
            KeyboardCode::Digit4 => "Digit4",
            KeyboardCode::Digit5 => "Digit5",
            KeyboardCode::Digit6 => "Digit6",
            KeyboardCode::Digit7 => "Digit7",
            KeyboardCode::Digit8 => "Digit8",
            KeyboardCode::Digit9 => "Digit9",
            KeyboardCode::KeyA => "KeyA",
            KeyboardCode::KeyB => "KeyB",
            KeyboardCode::KeyC => "KeyC",
            KeyboardCode::KeyD => "KeyD",
            KeyboardCode::KeyE => "KeyE",
            KeyboardCode::KeyF => "KeyF",
            KeyboardCode::KeyG => "KeyG",
            KeyboardCode::KeyH => "KeyH",
            KeyboardCode::KeyI => "KeyI",
            KeyboardCode::KeyJ => "KeyJ",
            KeyboardCode::KeyK => "KeyK",
            KeyboardCode::KeyL => "KeyL",
            KeyboardCode::KeyM => "KeyM",
            KeyboardCode::KeyN => "KeyN",
            KeyboardCode::KeyO => "KeyO",
            KeyboardCode::KeyP => "KeyP",
            KeyboardCode::KeyQ => "KeyQ",
            KeyboardCode::KeyR => "KeyR",
            KeyboardCode::KeyS => "KeyS",
            KeyboardCode::KeyT => "KeyT",
            KeyboardCode::KeyU => "KeyU",
            KeyboardCode::KeyV => "KeyV",
            KeyboardCode::KeyW => "KeyW",
            KeyboardCode::KeyX => "KeyX",
            KeyboardCode::KeyY => "KeyY",
            KeyboardCode::KeyZ => "KeyZ",

            // Numpad keys
            KeyboardCode::Numpad0 => "Numpad0",
            KeyboardCode::Numpad1 => "Numpad1",
            KeyboardCode::Numpad2 => "Numpad2",
            KeyboardCode::Numpad3 => "Numpad3",
            KeyboardCode::Numpad4 => "Numpad4",
            KeyboardCode::Numpad5 => "Numpad5",
            KeyboardCode::Numpad6 => "Numpad6",
            KeyboardCode::Numpad7 => "Numpad7",
            KeyboardCode::Numpad8 => "Numpad8",
            KeyboardCode::Numpad9 => "Numpad9",
            KeyboardCode::NumpadAdd => "NumpadAdd",
            KeyboardCode::NumpadSubtract => "NumpadSubtract",
            KeyboardCode::NumpadMultiply => "NumpadMultiply",
            KeyboardCode::NumpadDivide => "NumpadDivide",
            KeyboardCode::NumpadEnter => "NumpadEnter",
            KeyboardCode::NumpadDecimal => "NumpadDecimal",
            KeyboardCode::NumpadEqual => "NumpadEqual",
            KeyboardCode::NumpadComma => "NumpadComma",

            // Symbol keys
            KeyboardCode::Backquote => "Backquote",
            KeyboardCode::BracketLeft => "BracketLeft",
            KeyboardCode::BracketRight => "BracketRight",
            KeyboardCode::Comma => "Comma",
            KeyboardCode::Period => "Period",
            KeyboardCode::Semicolon => "Semicolon",
            KeyboardCode::Quote => "Quote",
            KeyboardCode::Backslash => "Backslash",
            KeyboardCode::Slash => "Slash",
            KeyboardCode::Minus => "Minus",
            KeyboardCode::Equal => "Equal",
            KeyboardCode::IntlBackslash => "IntlBackslash",
            KeyboardCode::IntlRo => "IntlRo",
            KeyboardCode::IntlYen => "IntlYen",

            // Media keys
            KeyboardCode::AudioVolumeDown => "AudioVolumeDown",
            KeyboardCode::AudioVolumeMute => "AudioVolumeMute",
            KeyboardCode::AudioVolumeUp => "AudioVolumeUp",
            KeyboardCode::BrowserBack => "BrowserBack",
            KeyboardCode::BrowserFavorites => "BrowserFavorites",
            KeyboardCode::BrowserForward => "BrowserForward",
            KeyboardCode::BrowserHome => "BrowserHome",
            KeyboardCode::BrowserRefresh => "BrowserRefresh",
            KeyboardCode::BrowserSearch => "BrowserSearch",
            KeyboardCode::BrowserStop => "BrowserStop",
            KeyboardCode::Eject => "Eject",
            KeyboardCode::LaunchApp1 => "LaunchApp1",
            KeyboardCode::LaunchApp2 => "LaunchApp2",
            KeyboardCode::LaunchMail => "LaunchMail",
            KeyboardCode::MediaPlayPause => "MediaPlayPause",
            KeyboardCode::MediaStop => "MediaStop",
            KeyboardCode::MediaTrackNext => "MediaTrackNext",
            KeyboardCode::MediaTrackPrevious => "MediaTrackPrevious",
            KeyboardCode::Power => "Power",
            KeyboardCode::Sleep => "Sleep",
            KeyboardCode::WakeUp => "WakeUp",

            // International keys
            KeyboardCode::Lang1 => "Lang1",
            KeyboardCode::Lang2 => "Lang2",
            KeyboardCode::Lang3 => "Lang3",
            KeyboardCode::Lang4 => "Lang4",
            KeyboardCode::Lang5 => "Lang5",
            KeyboardCode::Convert => "Convert",
            KeyboardCode::NonConvert => "NonConvert",
            KeyboardCode::KanaMode => "KanaMode",

            // Fallback
            KeyboardCode::Unidentified => "Unidentified",
        }
    }

    /// Check if this is a navigation key
    pub fn is_navigation(&self) -> bool {
        matches!(self,
            KeyboardCode::ArrowDown |
            KeyboardCode::ArrowLeft |
            KeyboardCode::ArrowRight |
            KeyboardCode::ArrowUp |
            KeyboardCode::Home |
            KeyboardCode::End |
            KeyboardCode::PageUp |
            KeyboardCode::PageDown
        )
    }

    /// Check if this is a modifier key
    pub fn is_modifier(&self) -> bool {
        matches!(self,
            KeyboardCode::AltLeft |
            KeyboardCode::AltRight |
            KeyboardCode::ControlLeft |
            KeyboardCode::ControlRight |
            KeyboardCode::ShiftLeft |
            KeyboardCode::ShiftRight |
            KeyboardCode::MetaLeft |
            KeyboardCode::MetaRight
        )
    }

    /// Check if this is a function key
    pub fn is_function_key(&self) -> bool {
        matches!(self,
            KeyboardCode::F1 | KeyboardCode::F2 | KeyboardCode::F3 | KeyboardCode::F4 |
            KeyboardCode::F5 | KeyboardCode::F6 | KeyboardCode::F7 | KeyboardCode::F8 |
            KeyboardCode::F9 | KeyboardCode::F10 | KeyboardCode::F11 | KeyboardCode::F12 |
            KeyboardCode::F13 | KeyboardCode::F14 | KeyboardCode::F15 | KeyboardCode::F16 |
            KeyboardCode::F17 | KeyboardCode::F18 | KeyboardCode::F19 | KeyboardCode::F20 |
            KeyboardCode::F21 | KeyboardCode::F22 | KeyboardCode::F23 | KeyboardCode::F24
        )
    }

    /// Check if this is a numpad key
    pub fn is_numpad(&self) -> bool {
        matches!(self,
            KeyboardCode::Numpad0 | KeyboardCode::Numpad1 | KeyboardCode::Numpad2 |
            KeyboardCode::Numpad3 | KeyboardCode::Numpad4 | KeyboardCode::Numpad5 |
            KeyboardCode::Numpad6 | KeyboardCode::Numpad7 | KeyboardCode::Numpad8 |
            KeyboardCode::Numpad9 | KeyboardCode::NumpadAdd | KeyboardCode::NumpadSubtract |
            KeyboardCode::NumpadMultiply | KeyboardCode::NumpadDivide | KeyboardCode::NumpadEnter |
            KeyboardCode::NumpadDecimal | KeyboardCode::NumpadEqual | KeyboardCode::NumpadComma
        )
    }
}

impl FromStr for KeyboardCode {
  type Err = ();

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      // Modifier keys
      "AltLeft" => Ok(KeyboardCode::AltLeft),
      "AltRight" => Ok(KeyboardCode::AltRight),
      "ControlLeft" => Ok(KeyboardCode::ControlLeft),
      "ControlRight" => Ok(KeyboardCode::ControlRight),
      "MetaLeft" => Ok(KeyboardCode::MetaLeft),
      "MetaRight" => Ok(KeyboardCode::MetaRight),
      "ShiftLeft" => Ok(KeyboardCode::ShiftLeft),
      "ShiftRight" => Ok(KeyboardCode::ShiftRight),

      // Whitespace keys
      "Enter" => Ok(KeyboardCode::Enter),
      "Tab" => Ok(KeyboardCode::Tab),
      "Space" => Ok(KeyboardCode::Space),

      // Navigation keys
      "ArrowDown" => Ok(KeyboardCode::ArrowDown),
      "ArrowLeft" => Ok(KeyboardCode::ArrowLeft),
      "ArrowRight" => Ok(KeyboardCode::ArrowRight),
      "ArrowUp" => Ok(KeyboardCode::ArrowUp),
      "End" => Ok(KeyboardCode::End),
      "Home" => Ok(KeyboardCode::Home),
      "PageDown" => Ok(KeyboardCode::PageDown),
      "PageUp" => Ok(KeyboardCode::PageUp),

      // UI keys
      "Escape" => Ok(KeyboardCode::Escape),
      "CapsLock" => Ok(KeyboardCode::CapsLock),
      "ScrollLock" => Ok(KeyboardCode::ScrollLock),
      "NumLock" => Ok(KeyboardCode::NumLock),
      "PrintScreen" => Ok(KeyboardCode::PrintScreen),
      "Pause" => Ok(KeyboardCode::Pause),
      "ContextMenu" => Ok(KeyboardCode::ContextMenu),

      // Function keys
      "F1" => Ok(KeyboardCode::F1),
      "F2" => Ok(KeyboardCode::F2),
      "F3" => Ok(KeyboardCode::F3),
      "F4" => Ok(KeyboardCode::F4),
      "F5" => Ok(KeyboardCode::F5),
      "F6" => Ok(KeyboardCode::F6),
      "F7" => Ok(KeyboardCode::F7),
      "F8" => Ok(KeyboardCode::F8),
      "F9" => Ok(KeyboardCode::F9),
      "F10" => Ok(KeyboardCode::F10),
      "F11" => Ok(KeyboardCode::F11),
      "F12" => Ok(KeyboardCode::F12),
      "F13" => Ok(KeyboardCode::F13),
      "F14" => Ok(KeyboardCode::F14),
      "F15" => Ok(KeyboardCode::F15),
      "F16" => Ok(KeyboardCode::F16),
      "F17" => Ok(KeyboardCode::F17),
      "F18" => Ok(KeyboardCode::F18),
      "F19" => Ok(KeyboardCode::F19),
      "F20" => Ok(KeyboardCode::F20),
      "F21" => Ok(KeyboardCode::F21),
      "F22" => Ok(KeyboardCode::F22),
      "F23" => Ok(KeyboardCode::F23),
      "F24" => Ok(KeyboardCode::F24),

      // Editing keys
      "Backspace" => Ok(KeyboardCode::Backspace),
      "Clear" => Ok(KeyboardCode::Clear),
      "Delete" => Ok(KeyboardCode::Delete),
      "Insert" => Ok(KeyboardCode::Insert),

      // Alphanumeric keys
      "Digit0" => Ok(KeyboardCode::Digit0),
      "Digit1" => Ok(KeyboardCode::Digit1),
      "Digit2" => Ok(KeyboardCode::Digit2),
      "Digit3" => Ok(KeyboardCode::Digit3),
      "Digit4" => Ok(KeyboardCode::Digit4),
      "Digit5" => Ok(KeyboardCode::Digit5),
      "Digit6" => Ok(KeyboardCode::Digit6),
      "Digit7" => Ok(KeyboardCode::Digit7),
      "Digit8" => Ok(KeyboardCode::Digit8),
      "Digit9" => Ok(KeyboardCode::Digit9),
      "KeyA" => Ok(KeyboardCode::KeyA),
      "KeyB" => Ok(KeyboardCode::KeyB),
      "KeyC" => Ok(KeyboardCode::KeyC),
      "KeyD" => Ok(KeyboardCode::KeyD),
      "KeyE" => Ok(KeyboardCode::KeyE),
      "KeyF" => Ok(KeyboardCode::KeyF),
      "KeyG" => Ok(KeyboardCode::KeyG),
      "KeyH" => Ok(KeyboardCode::KeyH),
      "KeyI" => Ok(KeyboardCode::KeyI),
      "KeyJ" => Ok(KeyboardCode::KeyJ),
      "KeyK" => Ok(KeyboardCode::KeyK),
      "KeyL" => Ok(KeyboardCode::KeyL),
      "KeyM" => Ok(KeyboardCode::KeyM),
      "KeyN" => Ok(KeyboardCode::KeyN),
      "KeyO" => Ok(KeyboardCode::KeyO),
      "KeyP" => Ok(KeyboardCode::KeyP),
      "KeyQ" => Ok(KeyboardCode::KeyQ),
      "KeyR" => Ok(KeyboardCode::KeyR),
      "KeyS" => Ok(KeyboardCode::KeyS),
      "KeyT" => Ok(KeyboardCode::KeyT),
      "KeyU" => Ok(KeyboardCode::KeyU),
      "KeyV" => Ok(KeyboardCode::KeyV),
      "KeyW" => Ok(KeyboardCode::KeyW),
      "KeyX" => Ok(KeyboardCode::KeyX),
      "KeyY" => Ok(KeyboardCode::KeyY),
      "KeyZ" => Ok(KeyboardCode::KeyZ),

      // Numpad keys
      "Numpad0" => Ok(KeyboardCode::Numpad0),
      "Numpad1" => Ok(KeyboardCode::Numpad1),
      "Numpad2" => Ok(KeyboardCode::Numpad2),
      "Numpad3" => Ok(KeyboardCode::Numpad3),
      "Numpad4" => Ok(KeyboardCode::Numpad4),
      "Numpad5" => Ok(KeyboardCode::Numpad5),
      "Numpad6" => Ok(KeyboardCode::Numpad6),
      "Numpad7" => Ok(KeyboardCode::Numpad7),
      "Numpad8" => Ok(KeyboardCode::Numpad8),
      "Numpad9" => Ok(KeyboardCode::Numpad9),
      "NumpadAdd" => Ok(KeyboardCode::NumpadAdd),
      "NumpadSubtract" => Ok(KeyboardCode::NumpadSubtract),
      "NumpadMultiply" => Ok(KeyboardCode::NumpadMultiply),
      "NumpadDivide" => Ok(KeyboardCode::NumpadDivide),
      "NumpadEnter" => Ok(KeyboardCode::NumpadEnter),
      "NumpadDecimal" => Ok(KeyboardCode::NumpadDecimal),
      "NumpadEqual" => Ok(KeyboardCode::NumpadEqual),
      "NumpadComma" => Ok(KeyboardCode::NumpadComma),

      // Symbol keys
      "Backquote" => Ok(KeyboardCode::Backquote),
      "BracketLeft" => Ok(KeyboardCode::BracketLeft),
      "BracketRight" => Ok(KeyboardCode::BracketRight),
      "Comma" => Ok(KeyboardCode::Comma),
      "Period" => Ok(KeyboardCode::Period),
      "Semicolon" => Ok(KeyboardCode::Semicolon),
      "Quote" => Ok(KeyboardCode::Quote),
      "Backslash" => Ok(KeyboardCode::Backslash),
      "Slash" => Ok(KeyboardCode::Slash),
      "Minus" => Ok(KeyboardCode::Minus),
      "Equal" => Ok(KeyboardCode::Equal),
      "IntlBackslash" => Ok(KeyboardCode::IntlBackslash),
      "IntlRo" => Ok(KeyboardCode::IntlRo),
      "IntlYen" => Ok(KeyboardCode::IntlYen),

      // Media keys
      "AudioVolumeDown" => Ok(KeyboardCode::AudioVolumeDown),
      "AudioVolumeMute" => Ok(KeyboardCode::AudioVolumeMute),
      "AudioVolumeUp" => Ok(KeyboardCode::AudioVolumeUp),
      "BrowserBack" => Ok(KeyboardCode::BrowserBack),
      "BrowserFavorites" => Ok(KeyboardCode::BrowserFavorites),
      "BrowserForward" => Ok(KeyboardCode::BrowserForward),
      "BrowserHome" => Ok(KeyboardCode::BrowserHome),
      "BrowserRefresh" => Ok(KeyboardCode::BrowserRefresh),
      "BrowserSearch" => Ok(KeyboardCode::BrowserSearch),
      "BrowserStop" => Ok(KeyboardCode::BrowserStop),
      "Eject" => Ok(KeyboardCode::Eject),
      "LaunchApp1" => Ok(KeyboardCode::LaunchApp1),
      "LaunchApp2" => Ok(KeyboardCode::LaunchApp2),
      "LaunchMail" => Ok(KeyboardCode::LaunchMail),
      "MediaPlayPause" => Ok(KeyboardCode::MediaPlayPause),
      "MediaStop" => Ok(KeyboardCode::MediaStop),
      "MediaTrackNext" => Ok(KeyboardCode::MediaTrackNext),
      "MediaTrackPrevious" => Ok(KeyboardCode::MediaTrackPrevious),
      "Power" => Ok(KeyboardCode::Power),
      "Sleep" => Ok(KeyboardCode::Sleep),
      "WakeUp" => Ok(KeyboardCode::WakeUp),

      // International keys
      "Lang1" => Ok(KeyboardCode::Lang1),
      "Lang2" => Ok(KeyboardCode::Lang2),
      "Lang3" => Ok(KeyboardCode::Lang3),
      "Lang4" => Ok(KeyboardCode::Lang4),
      "Lang5" => Ok(KeyboardCode::Lang5),
      "Convert" => Ok(KeyboardCode::Convert),
      "NonConvert" => Ok(KeyboardCode::NonConvert),
      "KanaMode" => Ok(KeyboardCode::KanaMode),

      // Unknown key
      _ => Ok(KeyboardCode::Unidentified),
    }
  }
}

/// MouseButton represents the different mouse buttons as defined by the
/// MouseEvent.button property in the DOM specification.
///
/// Values correspond to:
/// - 0: Main button (usually left)
/// - 1: Auxiliary button (usually middle/wheel)
/// - 2: Secondary button (usually right)
/// - 3: Fourth button (usually "Browser Back")
/// - 4: Fifth button (usually "Browser Forward")
///
/// See: https://developer.mozilla.org/en-US/docs/Web/API/MouseEvent/button
#[ derive( Debug, Clone, Copy, PartialEq, Eq, Hash, EnumCount ) ]
pub enum MouseButton
{
  Main,        // Left button (0)
  Auxiliary,   // Middle button (1)
  Secondary,   // Right button (2)
  Back,        // Back button (3)
  Forward,     // Forward button (4)
  Unknown( i16 ), // For any other values
}

impl MouseButton
{
  /// Convert a numeric button value to the corresponding MouseButton enum variant
  pub fn from_button( button : i16 ) -> Self
  {
    match button
    {
      0 => MouseButton::Main,
      1 => MouseButton::Auxiliary,
      2 => MouseButton::Secondary,
      3 => MouseButton::Back,
      4 => MouseButton::Forward,
      other => MouseButton::Unknown( other ),
    }
  }

  /// Convert a string representation to the corresponding MouseButton enum variant
  pub fn from_name( name : &str ) -> Self
  {
    MouseButton::from_str( name ).unwrap_or( MouseButton::Unknown( -1 ) )
  }

  /// Get the numeric button value for this MouseButton
  pub fn button_value( &self ) -> i16
  {
    match self
    {
      MouseButton::Main => 0,
      MouseButton::Auxiliary => 1,
      MouseButton::Secondary => 2,
      MouseButton::Back => 3,
      MouseButton::Forward => 4,
      MouseButton::Unknown( val ) => *val,
    }
  }

  /// Get a user-friendly name for this MouseButton
  pub fn name( &self ) -> &'static str
  {
    match self
    {
      MouseButton::Main => "Left",
      MouseButton::Auxiliary => "Middle",
      MouseButton::Secondary => "Right",
      MouseButton::Back => "Back",
      MouseButton::Forward => "Forward",
      MouseButton::Unknown( _ ) => "Unknown",
    }
  }

  /// Get the technical name for this MouseButton
  pub fn technical_name( &self ) -> &'static str
  {
    match self
    {
      MouseButton::Main => "Main",
      MouseButton::Auxiliary => "Auxiliary",
      MouseButton::Secondary => "Secondary",
      MouseButton::Back => "Back",
      MouseButton::Forward => "Forward",
      MouseButton::Unknown( _ ) => "Unknown",
    }
  }

  /// Check if this is the main (usually left) button
  pub fn is_main( &self ) -> bool
  {
    matches!( self, MouseButton::Main )
  }

  /// Check if this is the secondary (usually right) button
  pub fn is_secondary( &self ) -> bool
  {
    matches!( self, MouseButton::Secondary )
  }

  /// Check if this is the auxiliary (usually middle/wheel) button
  pub fn is_auxiliary( &self ) -> bool
  {
    matches!( self, MouseButton::Auxiliary )
  }

  /// Check if this is a navigation button (Back/Forward)
  pub fn is_navigation( &self ) -> bool
  {
    matches!( self, MouseButton::Back | MouseButton::Forward )
  }
}

impl FromStr for MouseButton
{
  type Err = ();

  fn from_str( s : &str ) -> Result< Self, Self::Err >
  {
    match s.to_lowercase().as_str()
    {
      "main" | "left" | "primary" => Ok( MouseButton::Main ),
      "auxiliary" | "middle" | "wheel" => Ok( MouseButton::Auxiliary ),
      "secondary" | "right" | "context" => Ok( MouseButton::Secondary ),
      "back" => Ok( MouseButton::Back ),
      "forward" => Ok( MouseButton::Forward ),
      _ => Err( () ),
    }
  }
}

impl From< i16 > for MouseButton
{
  fn from( value : i16 ) -> Self
  {
    MouseButton::from_button( value )
  }
}

# 🎮 browser_input

> **Ergonomic input handling for WebAssembly applications in the browser**

A lightweight, efficient input handling library specifically designed for Rust WebAssembly applications. Provides a unified interface for capturing and processing keyboard, mouse, and other input events in web browsers.

## ✨ Features

### 🎯 **Input Types**
- ✅ **Keyboard Input** - Full keyboard event capture with modifier support
- ✅ **Mouse Buttons** - Left, right, middle mouse button handling
- ✅ **Mouse Movement** - Precise pointer position and movement tracking
- ❌ **Mouse Wheel** - Scroll events (planned)

### 🛠️ **Core Capabilities**
- **Event Queue System** - Buffered input events with modifier key states
- **State Tracking** - Real-time key and button state queries
- **Modifier Support** - Alt, Ctrl, Shift key combination detection
- **Browser Integration** - Direct DOM event binding with WebAssembly
- **Memory Efficient** - Minimal overhead for web deployment

## 🚀 Quick Start

### Add to Your Project
```toml
[dependencies]
browser_input = { workspace = true, features = ["enabled"] }
```

### Basic Input Loop
```rust
use browser_input::*;

let mut input = Input::new( None, CLIENT );

loop
{
  // Update input state from DOM events
  input.update_state();

  // Process event queue
  for Event { event_type, alt, ctrl, shift } in input.event_queue().as_slice()
  {
    match event_type
    {
      EventType::MouseButton( button, Action::Press ) =>
      {
        println!( "Mouse button {:?} pressed with modifiers: Alt={}, Ctrl={}, Shift={}", 
                 button, alt, ctrl, shift );
      }
      EventType::KeyBoard( key, Action::Press ) =>
      {
        println!( "Key {:?} pressed", key );
      }
      _ => {}
    }
  }

  // Direct state queries
  let pointer_pos = input.pointer_position();
  let is_w_down = input.is_key_down( keyboard::KeyboardKey::KeyW );
  let is_space_down = input.is_key_down( keyboard::KeyboardKey::Space );

  if is_w_down && ctrl {
    // Handle Ctrl+W combination
  }

  // CRITICAL: Clear events at end of frame
  input.clear_events();
}
```

### Game Input Example
```rust
use browser_input::*;

struct GameInput
{
  input: Input,
  player_speed: f32,
}

impl GameInput
{
  pub fn new() -> Self
  {
    Self
    {
      input: Input::new( None, CLIENT ),
      player_speed: 5.0,
    }
  }

  pub fn update( &mut self ) -> PlayerMovement
  {
    self.input.update_state();
    
    let mut movement = PlayerMovement::default();
    
    // WASD movement
    if self.input.is_key_down( keyboard::KeyboardKey::KeyW ) {
      movement.forward = self.player_speed;
    }
    if self.input.is_key_down( keyboard::KeyboardKey::KeyS ) {
      movement.backward = self.player_speed;
    }
    if self.input.is_key_down( keyboard::KeyboardKey::KeyA ) {
      movement.left = self.player_speed;
    }
    if self.input.is_key_down( keyboard::KeyboardKey::KeyD ) {
      movement.right = self.player_speed;
    }
    
    // Mouse look
    movement.mouse_pos = self.input.pointer_position();
    
    // Process discrete events
    for event in self.input.event_queue().as_slice()
    {
      if let EventType::MouseButton( MouseButton::Left, Action::Press ) = event.event_type
      {
        movement.shoot = true;
      }
    }
    
    self.input.clear_events();
    movement
  }
}

#[ derive( Default ) ]
struct PlayerMovement
{
  forward: f32,
  backward: f32,
  left: f32,
  right: f32,
  mouse_pos: ( f32, f32 ),
  shoot: bool,
}
```

## 📚 API Reference

### Core Types

#### `Input`
Main input manager struct:
```rust
impl Input
{
  // Create new input handler
  pub fn new( canvas: Option< web_sys::HtmlCanvasElement >, client: ClientType ) -> Self;
  
  // Update state from DOM events (call once per frame)
  pub fn update_state( &mut self );
  
  // Get current event queue
  pub fn event_queue( &self ) -> &Vec< Event >;
  
  // Clear event queue (call at end of frame)
  pub fn clear_events( &mut self );
  
  // Query key state
  pub fn is_key_down( &self, key: keyboard::KeyboardKey ) -> bool;
  
  // Get current pointer position
  pub fn pointer_position( &self ) -> ( f32, f32 );
}
```

#### `Event`
Input event with modifier keys:
```rust
pub struct Event
{
  pub event_type: EventType,
  pub alt: bool,
  pub ctrl: bool,
  pub shift: bool,
}
```

#### `EventType`
Different types of input events:
```rust
pub enum EventType
{
  MouseButton( MouseButton, Action ),
  KeyBoard( keyboard::KeyboardKey, Action ),
  MouseMove( f32, f32 ), // x, y coordinates
}
```

### Key Constants

#### Keyboard Keys
```rust
use browser_input::keyboard::KeyboardKey;

// Movement keys
KeyboardKey::KeyW
KeyboardKey::KeyA
KeyboardKey::KeyS
KeyboardKey::KeyD

// Arrow keys
KeyboardKey::ArrowUp
KeyboardKey::ArrowDown
KeyboardKey::ArrowLeft
KeyboardKey::ArrowRight

// Action keys
KeyboardKey::Space
KeyboardKey::Enter
KeyboardKey::Escape
KeyboardKey::Tab

// Number keys
KeyboardKey::Digit1
KeyboardKey::Digit2
// ... etc
```

#### Mouse Buttons
```rust
use browser_input::MouseButton;

MouseButton::Left
MouseButton::Right
MouseButton::Middle
```

## 🎯 Usage Patterns

### Real-Time Controls
For games and interactive applications requiring immediate response:
```rust
// Query current state directly
let move_up = input.is_key_down( KeyboardKey::KeyW );
let move_down = input.is_key_down( KeyboardKey::KeyS );
let shooting = input.is_key_down( KeyboardKey::Space );
```

### Event-Driven Actions  
For UI interactions and discrete actions:
```rust
for event in input.event_queue().as_slice()
{
  match event.event_type
  {
    EventType::KeyBoard( KeyboardKey::Enter, Action::Press ) => {
      // Handle menu selection
    }
    EventType::MouseButton( MouseButton::Left, Action::Release ) => {
      // Handle button click
    }
    _ => {}
  }
}
```

### Modifier Key Combinations
```rust
for Event { event_type, ctrl, alt, shift } in input.event_queue().as_slice()
{
  if let EventType::KeyBoard( KeyboardKey::KeyS, Action::Press ) = event_type
  {
    if *ctrl {
      save_file(); // Ctrl+S
    } else if *alt {
      save_as(); // Alt+S
    } else {
      // Regular 'S' press
    }
  }
}
```

## ⚠️ Important Notes

### Frame Management
- **Always call `update_state()`** at the beginning of your main loop
- **Always call `clear_events()`** at the end of your main loop
- Failure to clear events will cause the event queue to grow indefinitely

### Canvas Binding
```rust
// Bind to specific canvas
let canvas = document.get_element_by_id( "game-canvas" )
  .unwrap()
  .dyn_into::< web_sys::HtmlCanvasElement >()
  .unwrap();
let input = Input::new( Some( canvas ), CLIENT );

// Bind to entire document (default)
let input = Input::new( None, CLIENT );
```

### Performance
- Event processing is O(n) where n is the number of events per frame
- State queries (`is_key_down`) are O(1) constant time
- Memory usage scales with the number of simultaneous key presses

## 🤝 Contributing

This crate is part of the CGTools workspace. Feel free to submit issues and pull requests on GitHub.

## 📄 License

MIT

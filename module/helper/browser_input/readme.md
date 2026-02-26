# browser_input

Ergonomic pointer and keyboard input handling for Rust WebAssembly applications.

Captures DOM events into a per-frame event queue and maintains a persistent input state. Works with mouse, touch, and stylus through the unified [Pointer Events API](https://developer.mozilla.org/en-US/docs/Web/API/Pointer_events), with full multi-touch support.

## Features

- **Unified pointer model** — mouse, touch, and stylus handled identically via `pointermove` / `pointerdown` / `pointerup` / `pointercancel`
- **Multi-touch tracking** — `active_pointers()` returns all current contacts as `(pointer_id, position)` pairs; use this for pinch-to-zoom or two-finger pan
- **Keyboard events** — full key press/release with modifier detection
- **Scroll wheel** — raw `delta_x / delta_y / delta_z` from the browser
- **Mobile-ready** — sets `touch-action: none` on the target element and calls `setPointerCapture` on every `pointerdown` so drag events keep firing even when a finger moves outside the element
- **Event queue + state polling** — process discrete events per frame, or query the current held-down state of any key or button

## Quick Start

```toml
[dependencies]
browser_input = { workspace = true }
```

```rust
use minwebgl as gl;
use browser_input::{ Input, CLIENT };
use gl::JsCast as _;

// Attach to a canvas element; coordinates are relative to the viewport.
let mut input = Input::new( Some( canvas.dyn_into().unwrap() ), CLIENT )?;

// Inside the RAF loop:
input.update_state();

for browser_input::Event { event_type, ctrl, .. } in input.event_queue().iter()
{
  match *event_type
  {
    browser_input::EventType::PointerButton( _id, pos, button, browser_input::Action::Press ) =>
    {
      // pos: I32x2 — use pos.0[0] for x, pos.0[1] for y
    }
    browser_input::EventType::KeyboardKey( browser_input::keyboard::KeyboardKey::Space, browser_input::Action::Press ) =>
    {
      // space pressed
    }
    browser_input::EventType::Wheel( delta ) =>
    {
      // delta.0[1] is delta_y (positive = scroll down)
    }
    _ => {}
  }
}

// Pinch-to-zoom with active_pointers
let pointers = input.active_pointers();
if pointers.len() >= 2
{
  let ( _, p0 ) = pointers[ 0 ];
  let ( _, p1 ) = pointers[ 1 ];
  // compute distance between p0 and p1 to derive scale delta
}

input.clear_events();
```

## API

### `Input::new( target, get_coords )`

Creates the handler and attaches DOM listeners. `target` is the `EventTarget` for pointer events (pointer, wheel); keyboard events always go on `document`. Pass `None` to also attach pointer events to `document`.

`get_coords` selects the coordinate space:

| Constant | Web API used | Relative to |
|----------|-------------|-------------|
| `CLIENT` | `clientX / clientY` | Viewport (canvas top-left when canvas fills the screen) |
| `PAGE`   | `pageX / pageY`     | Full page including scrolled-out area |
| `SCREEN` | `screenX / screenY` | Physical screen |

### Per-frame pattern

```rust
input.update_state();   // apply queued events to internal state
// ... read event_queue() and active_pointers() ...
input.clear_events();   // discard processed events
```

### Event types

```rust
pub enum EventType
{
  // pointer_id, position, button, press/release
  PointerButton( i32, I32x2, MouseButton, Action ),
  // pointer_id, new position (fires for every active contact)
  PointerMove( i32, I32x2 ),
  // key code, press/release
  KeyboardKey( KeyboardKey, Action ),
  // (delta_x, delta_y, delta_z)
  Wheel( F64x3 ),
  // pointer_id only — coordinates are unreliable for cancel events
  PointerCancel( i32 ),
}
```

`I32x2` coordinates are accessed as `pos.0[0]` (x) and `pos.0[1]` (y).

### State queries

```rust
input.is_key_down( KeyboardKey::KeyW )          // -> bool
input.is_button_down( MouseButton::Main )        // -> bool  (tracks last press/release; not reference-counted across multiple touches)
input.pointer_position()                         // -> I32x2   (last moved pointer; non-deterministic with multiple touches)
input.active_pointers()                          // -> &[(i32, I32x2)]
input.scroll()                                   // -> &F64x3  (accumulated wheel delta since last clear_events)
```

### Mouse buttons

```rust
MouseButton::Main        // left / primary touch
MouseButton::Secondary   // right
MouseButton::Auxiliary   // middle
```

### Cleanup

Listeners are removed automatically when `Input` is dropped.

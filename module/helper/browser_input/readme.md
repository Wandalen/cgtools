# browser_input

Implement functionality to retrieve user input on browser page.

## Implemented Features

- ✅ Keyboard keys input
- ✅ Mouse buttons input
- ✅ Mouse movement
- ❌ Mouse wheel

## Example

``` rust
let mut input = browser_input::Input::new( None, browser_input::CLIENT );

loop
{
  input.update_state();

  for browser_input::Event { event_type, alt, ctrl, shift } in input.event_queue().as_slice()
  {
    if let browser_input::EventType::MouseButton( button, browser_input::Action::Press ) = event_type
    {
      // ...
    }
  }

  let pointer_position = input.pointer_position();
  let is_w_down = input.is_key_down( browser_input::keyboard::KeyboardKey::KeyW );

  // THIS IS ESSENTIAL, don't forget to do it at the end of loop iteration
  input.clear_events();
}
```

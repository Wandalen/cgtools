# Manual Testing Guide for browser_input

This guide provides procedures for manually testing the `browser_input` crate's pointer event handling functionality. Manual testing is essential for verifying touch and pointer interactions that cannot be easily automated.

## Prerequisites

- A touch-enabled device (smartphone, tablet, or touch screen monitor) for touch testing
- A mouse and keyboard for pointer and keyboard testing
- A device with a touchpad or trackpad for scroll testing
- A modern web browser that supports the W3C Pointer Events API

## Test Application

Use the `examples/minwebgl/touch_input_test` example for manual testing:

```bash
cd examples/minwebgl/touch_input_test
npm install
npm run dev
```

Open the provided local server URL in your browser (e.g., `http://localhost:8080`).

## Test Scenarios

### 1. Single-Finger Touch (Mobile/Tablet)

**Objective:** Verify single pointer tracking and touch-action prevention.

**Steps:**
1. Open the test application on a touch device
2. Touch the canvas with one finger
3. Move your finger across the canvas
4. Lift your finger

**Expected Behavior:**
- The application displays the pointer position
- Touch gestures (scroll, zoom) are prevented on the canvas (touch-action: none)
- Pointer ID and position are tracked correctly
- `pointerdown`, `pointermove`, and `pointerup` events are received
- Active pointers list shows one entry while touching

### 2. Two-Finger Touch (Multi-Touch)

**Objective:** Verify multi-touch pointer tracking for gestures like pinch-to-zoom.

**Steps:**
1. Touch the canvas with two fingers simultaneously
2. Move both fingers (e.g., pinch gesture)
3. Lift one finger, then the other

**Expected Behavior:**
- Both pointers are tracked independently with unique pointer IDs
- `active_pointers()` contains two entries while both fingers are down
- Each finger's position updates independently
- Pointer count decreases correctly as fingers are lifted
- No duplicate pointer IDs in the active pointers list

### 3. Pointer Cancel Event

**Objective:** Verify handling of interrupted pointer events.

**Steps:**
1. Touch the canvas with one finger
2. While holding the finger down, trigger a pointer cancel event by:
   - Locking the device screen (if on mobile)
   - Opening the browser's address bar or menu
   - Rotating the device orientation
   - Switching to another app

**Expected Behavior:**
- `pointercancel` event is received
- The interrupted pointer is removed from `active_pointers()`
- If all pointers are cancelled, all mouse buttons are reset to unpressed state
- Application state remains consistent after cancel

### 4. Mouse Click and Drag (Desktop)

**Objective:** Verify mouse button and movement tracking.

**Steps:**
1. Click the left mouse button on the canvas
2. Hold and drag the mouse
3. Release the mouse button
4. Repeat with right and middle mouse buttons

**Expected Behavior:**
- Mouse button press is detected (`is_button_down()` returns true)
- Pointer position updates during drag
- Pointer capture ensures events continue even if cursor leaves canvas
- Mouse button release is detected
- `active_pointers()` has one entry while mouse button is held

### 5. Scroll Wheel

**Objective:** Verify scroll accumulation and reset.

**Steps:**
1. Scroll the mouse wheel up and down on the canvas
2. Use trackpad two-finger scroll if available
3. Call `clear_events()` and check scroll value

**Expected Behavior:**
- Scroll delta accumulates in `scroll()` value
- Scroll direction is correct (positive/negative)
- `clear_events()` resets the scroll accumulator to zero

### 6. Keyboard Input

**Objective:** Verify keyboard event handling.

**Steps:**
1. Focus the browser window with the test application
2. Press and release various keys (letters, numbers, arrow keys, modifiers)
3. Hold down Ctrl, Alt, or Shift while pressing other keys

**Expected Behavior:**
- Key press events are captured
- Key release events are captured
- Modifier key states (Ctrl, Alt, Shift) are tracked correctly
- `is_key_down()` returns true while key is held

### 7. Excessive Pointer Flood (DoS Protection)

**Objective:** Verify the MAX_ACTIVE_POINTERS limit prevents unbounded memory growth.

**Steps:**
1. Open the browser console while running the test application
2. Execute the following JavaScript to simulate a flood of synthetic pointer events:
```javascript
const canvas = document.querySelector('canvas');
for (let i = 0; i < 100; i++) {
  canvas.dispatchEvent(new PointerEvent('pointerdown', {
    pointerId: i,
    clientX: 100,
    clientY: 100,
    bubbles: true
  }));
}
```

**Expected Behavior:**
- The application remains stable and responsive
- `active_pointers()` length is capped at 32 (MAX_ACTIVE_POINTERS)
- No memory exhaustion or crash occurs
- Additional pointers beyond the limit are ignored

## Test Matrix

| Scenario | Desktop Mouse | Touch (1 finger) | Touch (2+ fingers) | Trackpad |
|----------|--------------|------------------|-------------------|----------|
| Single pointer tracking | ✓ | ✓ | ✓ | ✓ |
| Multi-touch tracking | N/A | N/A | ✓ | N/A |
| Pointer cancel | ✓ | ✓ | ✓ | ✓ |
| Scroll wheel | ✓ | N/A | N/A | ✓ |
| Keyboard events | ✓ | ✓ | ✓ | ✓ |
| DoS protection | ✓ | ✓ | ✓ | ✓ |

## Reporting Issues

When reporting issues found during manual testing, please include:
- Device type and operating system
- Browser name and version
- Detailed steps to reproduce
- Expected vs. actual behavior
- Screenshots or screen recordings if applicable
- Browser console errors (if any)

## Additional Notes

- The W3C Pointer Events API unifies mouse, touch, and pen input into a single event model
- `setPointerCapture` is automatically called on `pointerdown` to ensure drag events continue even when the pointer leaves the target element
- The `touch-action: none` CSS property is automatically set on the pointer event target to prevent browser gesture interference
- Pointer IDs are unique for each active pointer contact (e.g., each finger on a touch screen)
- On desktop, mouse interactions typically result in a single active pointer with ID 1

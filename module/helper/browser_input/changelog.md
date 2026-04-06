# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

## [0.3.0] - 2026-02-20

### Breaking Changes
- Renamed `EventType::MouseButton` to `EventType::PointerButton` to reflect unified pointer model
- Renamed `EventType::MouseMovement` to `EventType::PointerMove` for consistency with W3C Pointer Events API
- Changed field structure for pointer events to include pointer ID for multi-touch support

### Added
- Added `EventType::PointerCancel` variant for handling interrupted pointer events
- Added `active_pointers()` method to track all currently active pointer contacts (multi-touch support)
- Automatic `setPointerCapture` on pointer target for consistent event delivery
- Automatic `touch-action: none` CSS property on pointer target to prevent browser gesture interference
- Public `State` struct and `apply_events_to_state()` function for testing

### Fixed
- Fixed scroll accumulator not resetting in `clear_events()`

### Migration Guide
When updating pattern matches on `EventType`:
- Replace `EventType::MouseButton(button, action)` with `EventType::PointerButton(pointer_id, pos, button, action)`
- Replace `EventType::MouseMovement(pos)` with `EventType::PointerMove(pointer_id, pos)`
- Handle new `EventType::PointerCancel(pointer_id)` variant where appropriate

## [0.1.0] - 2024-08-08

### Added
- Initial release of browser_input crate
- Ergonomic input handling for WebAssembly applications
- Keyboard event processing and key mapping
- Mouse event handling with position tracking
- Pointer event support for modern browsers
- Input state management and utilities
- WebAssembly-optimized design for browser environments

[0.3.0]: https://github.com/Wandalen/cgtools/releases/tag/browser_input-v0.3.0
[0.1.0]: https://github.com/Wandalen/cgtools/releases/tag/browser_input-v0.1.0
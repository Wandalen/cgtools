# 📝 browser_log

> **Advanced logging and panic handling for WebAssembly applications**

A specialized logging utility designed for Rust WebAssembly applications running in browsers and Node.js environments. Seamlessly integrates with JavaScript's console API while providing enhanced debugging capabilities and panic handling for WASM applications.

## ✨ Features

### 📊 **Console Integration**
- **Multi-Level Logging** - Debug, info, warn, error logging levels
- **JavaScript Console API** - Direct integration with browser developer tools
- **Formatted Output** - Structured log messages with timestamps and context
- **Performance Logging** - Timing and performance measurement utilities

### 🔧 **Panic Management**
- **Panic Interception** - Capture Rust panics in WASM environment
- **Console Error Output** - Redirect panic messages to console.error
- **Stack Trace Preservation** - Maintain debugging information across WASM boundary
- **Graceful Error Handling** - Prevent silent failures in web applications

### 🌐 **Cross-Platform**
- **Browser Support** - All modern browsers with WebAssembly support
- **Node.js Compatible** - Server-side WebAssembly applications
- **Development Tools** - Enhanced debugging experience in DevTools
- **Production Ready** - Configurable log levels for deployment

## 📦 Installation

Add to your `Cargo.toml`:
```toml
browser_log = { workspace = true }
```

## 🚀 Quick Start

### Basic Logging Setup

```rust
use browser_log::*;

// Initialize logging (call once at startup)
fn init_logging() {
  // Setup panic handler
  panic::setup(panic::Config::default());
  
  // Initialize logger with default settings
  log::setup::setup(log::setup::Config::default());
}

// Use standard Rust logging macros
fn example_logging() {
  ::log::info!("Application started");
  ::log::debug!("Debug information: {}", 42);
  ::log::warn!("Warning message");
  ::log::error!("Error occurred: {}", "connection failed");
}
```

### Advanced Configuration

```rust
use browser_log::*;
use browser_log::log::console;

fn setup_advanced_logging() {
  // Custom panic handler configuration
  let config = panic::Config {
    with_location: true,
    with_stack_trace: true,
  };
  panic::setup(config);
  
  // Configure logger with specific level
  log::setup::setup(log::setup::Config::new(::log::Level::Debug));
  
  // Performance timing (basic console timing)
  console::time();
  // ... perform operation
  console::time_end();
}
```

## 📖 API Reference

### Core Functions

| Function | Purpose | Example |
|----------|---------|---------|
| `log::setup::setup()` | Initialize logger | `browser_log::log::setup::setup(Default::default())` |
| `panic::setup()` | Setup panic handling | `browser_log::panic::setup(Default::default())` |
| `console::log_1()` | Direct console output | `console::log_1(&JsValue::from_str("message"))` |
| `console::time()` | Performance timing | `console::time()` |

### Logging Levels

```rust
// Standard Rust logging levels work seamlessly
log::trace!("Detailed tracing information");
log::debug!("Development debugging info");
log::info!("General information");
log::warn!("Warning conditions");
log::error!("Error conditions");
```

### Console API

```rust,no_run
use browser_log::log::console;
use wasm_bindgen::JsValue;

// Direct console methods (using web-sys console API)
console::log_1(&JsValue::from_str("Basic log message"));
console::info_1(&JsValue::from_str("Information message"));
console::warn_1(&JsValue::from_str("Warning message"));
console::error_1(&JsValue::from_str("Error message"));

// Performance timing
console::time();
// ... perform database query
console::time_end();

// Grouped logging
console::group_1(&JsValue::from_str("User Actions"));
console::log_1(&JsValue::from_str("User clicked button"));
console::log_1(&JsValue::from_str("Form submitted"));
console::group_end();
```

## 🎯 Use Cases

### Web Application Development
- **Frontend Debugging** - Real-time debugging in browser DevTools
- **Error Tracking** - Capture and log runtime errors
- **Performance Monitoring** - Measure operation timing and performance
- **User Activity Logging** - Track user interactions and application state

### Game Development
- **Game State Debugging** - Log game mechanics and state changes
- **Performance Profiling** - Monitor frame rates and render timing
- **Asset Loading** - Track resource loading progress
- **Player Action Logging** - Debug player input and game responses

### Scientific Computing
- **Algorithm Debugging** - Trace complex calculation steps
- **Data Processing** - Log data transformation pipelines
- **Visualization** - Debug rendering and graphics operations
- **Research Logging** - Record experimental parameters and results

## 🔧 Advanced Features

### Custom Panic Handlers

```rust
use browser_log::*;
use std::panic;

// Custom panic handler with user notification
panic::set_hook(Box::new(|panic_info| {
  let message = match panic_info.payload().downcast_ref::<&str>() {
    Some(s) => *s,
    None => "Unknown panic occurred",
  };
  
  // Use the browser_log panic handler
  browser_log::panic::hook(panic_info, &browser_log::panic::Config::default());
  
  // Show user-friendly error message (requires Window feature)
  // web_sys::window().unwrap().alert_with_message(&format!("An error occurred: {}", message));
}));
```

### Conditional Logging

```rust
// Only log in debug builds
#[cfg(debug_assertions)]
fn debug_log(message: &str) {
  ::log::debug!("{}", message);
}

// Log with context information
fn log_with_context(operation: &str, data: &impl std::fmt::Debug) {
  ::log::info!("[{}] Data: {:?}", operation, data);
}
```

### Performance Profiling

```rust
use browser_log::log::console;

struct ProfileScope {
  _name: String,
}

impl ProfileScope {
  fn new(name: &str) -> Self {
    console::time();
    Self { _name: name.to_string() }
  }
}

impl Drop for ProfileScope {
  fn drop(&mut self) {
    console::time_end();
  }
}

// Usage
fn expensive_operation() {
  let _profile = ProfileScope::new("expensive_operation");
  // ... perform work
} // Automatically logs timing when dropped
```

## ⚡ Best Practices

### Production Deployment
- Use conditional compilation for debug logs
- Set appropriate log levels for production
- Consider log message size impact on bundle size
- Implement log throttling for high-frequency events

### Development Workflow
- Use structured logging with consistent formatting
- Include context information in error messages
- Group related log messages for better organization
- Leverage browser DevTools filtering and search capabilities

## 🔧 Integration with Other Tools

### With wasm-pack
```toml
[dependencies]
browser_log = { workspace = true }
console_error_panic_hook = "0.1"
```

### With web frameworks
The logger integrates seamlessly with popular Rust web frameworks like Yew, Seed, and others that compile to WebAssembly.

## 📊 Technical Details

### WebAssembly Compatibility
- Optimized for `wasm32-unknown-unknown` target
- Minimal overhead for high-performance applications
- Compatible with all WebAssembly runtimes
- Thread-safe for multi-threaded WASM applications (when supported)

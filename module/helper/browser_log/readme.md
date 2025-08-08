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
  init_panic_handler();
  
  // Initialize logger with default settings
  init_logger().expect("Failed to initialize logger");
}

// Use standard Rust logging macros
fn example_logging() {
  log::info!("Application started");
  log::debug!("Debug information: {}", 42);
  log::warn!("Warning message");
  log::error!("Error occurred: {}", "connection failed");
}
```

### Advanced Configuration

```rust
use browser_log::*;

fn setup_advanced_logging() {
  // Custom panic handler with additional context
  init_panic_handler_with_context(|info| {
    console::error(&format!("PANIC: {}\nLocation: {:?}", info, info.location()));
  });
  
  // Configure logger with specific level
  init_logger_with_level(log::Level::Debug).expect("Logger init failed");
  
  // Performance timing
  let timer = console::time("operation");
  // ... perform operation
  timer.end();
}
```

## 📖 API Reference

### Core Functions

| Function | Purpose | Example |
|----------|---------|---------|
| `init_logger()` | Initialize default logger | `browser_log::init_logger()?` |
| `init_panic_handler()` | Setup panic handling | `browser_log::init_panic_handler()` |
| `console::log()` | Direct console output | `console::log("message")` |
| `console::time()` | Performance timing | `let timer = console::time("test")` |

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

```rust
use browser_log::console;

// Direct console methods
console::log("Basic log message");
console::info("Information message");
console::warn("Warning message");
console::error("Error message");

// Performance timing
let timer = console::time("database_query");
// ... perform database query
timer.end(); // Outputs timing to console

// Grouped logging
console::group("User Actions");
console::log("User clicked button");
console::log("Form submitted");
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

// Custom panic handler with user notification
init_panic_handler_with_context(|panic_info| {
  let message = match panic_info.payload().downcast_ref::<&str>() {
    Some(s) => *s,
    None => "Unknown panic occurred",
  };
  
  console::error(&format!("Application Error: {}", message));
  
  // Show user-friendly error message
  web_sys::window()
    .unwrap()
    .alert_with_message(&format!("An error occurred: {}", message));
});
```

### Conditional Logging

```rust
use browser_log::*;

// Only log in debug builds
#[cfg(debug_assertions)]
fn debug_log(message: &str) {
  log::debug!("{}", message);
}

// Log with context information
fn log_with_context(operation: &str, data: &impl std::fmt::Debug) {
  log::info!("[{}] Data: {:?}", operation, data);
}
```

### Performance Profiling

```rust
use browser_log::console;

struct ProfileScope {
  name: String,
}

impl ProfileScope {
  fn new(name: &str) -> Self {
    console::time(name);
    Self { name: name.to_string() }
  }
}

impl Drop for ProfileScope {
  fn drop(&mut self) {
    console::time_end(&self.name);
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

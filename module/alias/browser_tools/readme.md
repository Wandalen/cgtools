# 🌐 browser_tools

**Essential Browser & WebAssembly Utilities**

A comprehensive toolkit for Rust applications running in WebAssembly environments. This crate bridges the gap between Rust code and browser APIs, providing seamless logging, panic handling, and debugging capabilities for web applications.

## 🎯 Features

- **Console Integration**: Direct integration with JavaScript's `console` API
- **Multi-Level Logging**: Support for various log levels (debug, info, warn, error)
- **Panic Handling**: Intelligent panic redirection to browser console
- **WebAssembly Optimized**: Specifically designed for `wasm32-unknown-unknown` target
- **Zero-Cost Abstractions**: Minimal overhead when logging is disabled
- **Browser & Node.js Compatible**: Works in both browser and server-side JavaScript environments

## 🚀 Quick Start

Add this to your `Cargo.toml`:

```toml
[dependencies]
browser_tools = "0.2"
```

## 💡 Usage

### Basic Logging

```rust,no_run
use browser_tools::*;

// Initialize logging (call once at startup)
log::setup::setup(log::setup::Config::default());

// Use standard logging macros
log::info!("Application started");
log::warn!("Performance warning");
log::error!("Something went wrong");
log::debug!("Detailed debugging info");
```

### Panic Handling

```rust,no_run
use browser_tools::*;

// Set up panic handler for better error reporting
panic::setup(panic::Config::default());

// Now panics will be visible in browser console
panic!("This will show up in console.error");
```

## 🔧 Core Components

### Logging System
- **Console Integration**: Messages appear directly in browser dev tools
- **Severity Levels**: Proper categorization of log messages
- **Performance Optimized**: Efficient message handling in WASM context

### Panic Handler
- **Error Visibility**: Panic messages redirected to `console.error`
- **Stack Traces**: Detailed error information when available
- **Debug Enhancement**: Improved debugging experience in web environments

## 🎮 Use Cases

- **WebGL Applications**: Debug rendering and graphics issues
- **WASM Games**: Monitor game state and performance
- **Web Tools**: Build browser-based development utilities
- **Interactive Demos**: Provide user-friendly error reporting

## ⚡ Features Configuration

The crate supports optional features for flexible integration:

```toml
[dependencies]
browser_tools = { version = "0.2", features = ["full"] }
```

Available features:
- `enabled`: Core logging functionality
- `default`: Standard configuration
- `full`: All available features

## 📖 Documentation

For detailed API documentation and examples, run:

```bash
cargo doc --open
```

## 🔗 Integration

This crate works seamlessly with other cgtools modules, particularly those targeting WebAssembly environments. It's commonly used alongside MinWebGL and MinWebGPU examples for comprehensive browser-based graphics applications.

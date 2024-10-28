# browser_log

Utility to log in wasm/browser.

### Logging

It integrates with JavaScript's `console` API to output log messages with varying levels of severity, enhancing the visibility and management of log data in web environments.

<!-- qqq : add concise example -->

### Panic Handling Mechanism

Also this crate provides a debugging utility for Rust applications compiled to WebAssembly (`wasm32-unknown-unknown`). It redirects panic messages to JavaScript's `console.error`, enhancing error visibility in web browsers and Node.js environments.

<!-- qqq : add concise example -->

## Installation

Add the following to your `Cargo.toml`:
```toml
[dependencies]
browser_log = "0.2"
```

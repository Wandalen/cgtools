# WASM build size optimization
A little optimization for decrease builded wasm file.

Results:
- WASM size before optimization: 90.4 KB
- wasm-opt stage: 56.1 KB
- WASM size after optimization: 56.1 KB

## How it works
### wasm-opt
Added line of code to *index.html*:
```html
<link data-trunk rel="rust" data-wasm-opt="s" />
```
When we build the example with *trunk* (in release mode), it uses *wasm-opt* with *s*-level optimization.

Size optimization levels of *wasm-opt*:
  - S-level: base size opt.
  - Z-level: most aggressively, but at further potential speed costs.

Shrink methods of *wasm-opt*:
  - removing redurant code
  - reducing size of instructions
  - minimizing the number of sections and metadata

### wee_alloc
*wee_alloc* crate is used - an allocator for WASM that reduces the binary size.
Set *wee_alloc* as global allocator instead of *std::alloc* as the latter is for general purpose.
```rust
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
```
The *wee_alloc* reducing the total amount of allocated memory used by the application and minimizing the use of redundant functions that may not be needed in Wasm environments.

## Building
```bash
trunk build --release
```

# Spinning cube
![](./spinning_cube.gif)

# WASM build size optimization

A little optimization for decrease builded wasm file.

Results:

- WASM size before optimization: 123.5 KB
- wee_alloc stage: 117.3 KB
- wasm-opt stage: 79,6 KB
- wasm-strip stage: 79,4 KB
- WASM size after optimization: 79.4 KB

## How it works

### wasm-opt

Added line of code to _index.html_:

```html
<link data-trunk rel="rust" data-wasm-opt="s" />
```

When we build the example with _trunk_ (in release mode), it uses _wasm-opt_ with _s_-level optimization.

Size optimization levels of _wasm-opt_:

- S-level: base size opt.
- Z-level: most aggressively, but at further potential speed costs.

Shrink methods of _wasm-opt_:

- removing redurant code
- reducing size of instructions
- minimizing the number of sections and metadata

### wasm-strip

### wee_alloc

_wee_alloc_ crate is used - an allocator for WASM that reduces the binary size.
Set _wee_alloc_ as global allocator instead of _std::alloc_ as the latter is for general purpose.

```rust
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
```

The _wee_alloc_ reducing the total amount of allocated memory used by the application and minimizing the use of redundant functions that may not be needed in Wasm environments.

## Building

```bash
make build
```

# Spinning triangle

![](./spinning_triangle.gif)

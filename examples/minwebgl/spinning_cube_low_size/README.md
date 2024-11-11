# WASM build size optimization
A little optimization for decrease builded wasm file.

Results:
- WASM size before optimization: 90.4 KB
- lol_alloc stage: 81.6 KB
- wasm-opt stage: 47.7 KB
- WASM size after optimization: 47.7 KB

## How it works
### wasm-opt
Added line of code to *index.html*:
```html
<link data-trunk rel="rust" />
```
When we build the example with *trunk* (in release mode), it uses *wasm-opt* with *s*-level optimization.

Size optimization levels of *wasm-opt*:
  - S-level: base size opt.
  - Z-level: most aggressively, but at further potential speed costs.

Shrink methods of *wasm-opt*:
  - removing redurant code
  - reducing size of instructions
  - minimizing the number of sections and metadata

### lol_alloc
*lol_alloc* crate is used - an allocator for WASM that reduces the binary size.
Set *lol_alloc* as global allocator instead of *std::alloc* as the latter is for general purpose.
```rust
#[ cfg( target_arch = "wasm32" ) ]
#[ global_allocator ]
static ALLOCATOR : lol_alloc::LeakingPageAllocator = lol_alloc::LeakingPageAllocator;
```
The *lol_alloc* reducing the total amount of allocated memory used by the application and minimizing the use of redundant functions that may not be needed in Wasm environments.

## Building
```bash
trunk build --release
```

# Spinning cube
![](./spinning_cube.gif)

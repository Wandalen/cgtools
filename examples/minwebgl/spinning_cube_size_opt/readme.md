# 📦 Spinning Cube Size Optimization

**WebAssembly Build Size Optimization Showcase**

A spinning cube demonstration that showcases effective techniques for reducing WASM bundle size. This example provides practical insights into optimizing WebAssembly builds for production deployment while maintaining visual quality and performance.

## 🎯 Showcase
![](./spinning_cube.gif)

## ✨ Value Proposition
Dramatically reduce WASM bundle size through strategic optimization techniques, making your web applications faster to load and more efficient to distribute.

## 📊 Optimization Results

| Stage | Bundle Size | Reduction |
|-------|-------------|-----------|
| **Before Optimization** | 90.4 KB | - |
| **After lol_alloc** | 81.6 KB | 9.7% |
| **After wasm-opt** | 47.7 KB | 47.2% |
| **Final Optimized** | **47.7 KB** | **🎉 47.2% smaller** |

## 🔧 Optimization Techniques

### 🛠️ wasm-opt Integration
Configure Trunk to use wasm-opt during build by adding to your `index.html`:

```html
<link data-trunk rel="rust" data-wasm-opt="s" />
```

When building with `trunk build --release`, this automatically applies size optimization.

**wasm-opt Optimization Levels:**
- **`-Os`**: Balanced size optimization (recommended for most cases)
- **`-Oz`**: Maximum size reduction (may impact performance)

**Optimization Techniques Applied:**
- 🗑️ Dead code elimination
- 🔄 Instruction size reduction
- 📦 Metadata and section minimization
- 🧹 Redundant function removal

### 🚀 lol_alloc Allocator
Replace the standard allocator with `lol_alloc`, a specialized WASM allocator designed for minimal binary size:

```rust
#[cfg(target_arch = "wasm32")]
#[global_allocator]
static ALLOCATOR: lol_alloc::LeakingPageAllocator = lol_alloc::LeakingPageAllocator;
```

**Benefits of lol_alloc:**
- 📦 **Smaller Binary Size**: Optimized specifically for WASM environments
- ⚡ **Reduced Overhead**: Eliminates unnecessary allocator features
- 🎯 **WASM-Targeted**: Designed for web deployment constraints
- 🧹 **Simplified Implementation**: Fewer allocator-related functions in binary

## 🚀 Building & Running

### Development Build
```bash
trunk serve
```

### Production Build (Optimized)
```bash
trunk build --release
```

The optimized build will automatically apply:
- lol_alloc allocator replacement
- wasm-opt size optimization
- Dead code elimination
- Asset minification

## 🎯 Key Learnings

- **Use lol_alloc** for WebAssembly targets to reduce allocator overhead
- **Configure wasm-opt** in your build pipeline for automatic optimization
- **Measure before and after** to quantify optimization impact
- **Balance size vs performance** based on your application needs

## 📖 Related Examples

- **[Minimize WASM](../minimize_wasm/)**: Additional size reduction techniques
- **[Spinning Cube](../spinning_cube/)**: Base implementation without optimizations

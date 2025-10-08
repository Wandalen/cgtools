# 📏 line_tools

**High-performance line rendering for WebGL applications**

A specialized library for rendering lines, curves, and vector graphics in WebGL. Optimized for real-time rendering with support for various line styles, anti-aliasing, and efficient batch processing. Perfect for data visualization, CAD applications, and interactive graphics.

## ✨ Features

### 📏 **Line Rendering**
- **Anti-Aliased Lines** - Smooth, high-quality line rendering
- **Variable Width Lines** - Support for dynamic line thickness
- **Line Caps & Joins** - Configurable line endings and connections
- **Dash Patterns** - Dashed and dotted line styles

### ✨ **Visual Quality**
- **Multi-Sample Anti-Aliasing** - Hardware-accelerated edge smoothing
- **Smooth Curves** - Bezier and spline curve rendering
- **Color Gradients** - Linear and radial gradient support
- **Alpha Blending** - Transparent and semi-transparent lines

### ⚡ **Performance**
- **Batch Rendering** - Efficient rendering of multiple lines
- **WebGL Optimized** - Direct GPU-accelerated rendering
- **Memory Efficient** - Minimal overhead for large datasets
- **Real-Time Updates** - Dynamic line modification support

### 🔧 **Integration**
- **ndarray_cg Integration** - Matrix transformation support
- **minwebgl Compatibility** - Seamless WebGL context integration
- **Serialization Support** - Save/load line data via serde

## 🚀 Installation

Add to your `Cargo.toml`:
```toml
line_tools = { workspace = true }
```

## 💡 Quick Start

### Basic Line Rendering

```rust,ignore
use line_tools::*;
use minwebgl as gl;
use ndarray_cg::*;

fn render_lines(gl: &gl::WebGl2RenderingContext) -> Result<(), Box<dyn std::error::Error>> {
  // Create line renderer
  let mut renderer = LineRenderer::new(gl)?;
  
  // Define line points
  let points = vec![
    [0.0, 0.0],   // Start point
    [1.0, 0.5],   // Control point
    [2.0, 0.0],   // End point
  ];
  
  // Render line with styling
  renderer.draw_line(&points)
    .width(2.0)
    .color([1.0, 0.0, 0.0, 1.0]) // Red color
    .antialias(true)
    .render(gl)?;
  
  Ok(())
}
```

### Advanced Line Styling

```rust,ignore
use line_tools::*;

fn styled_lines(renderer: &mut LineRenderer, gl: &WebGl2RenderingContext) -> Result<(), Box<dyn std::error::Error>> {
  // Dashed line
  let line_points = vec![[0.0, 0.0], [1.0, 1.0]];
  renderer.draw_line(&line_points)
    .width(3.0)
    .dash_pattern(&[10.0, 5.0, 2.0, 5.0])
    .color([0.0, 1.0, 0.0, 1.0])
    .render(gl)?;
  
  // Gradient line
  let curve_points = vec![[0.0, 0.0], [0.5, 1.0], [1.0, 0.0]];
  renderer.draw_line(&curve_points)
    .width(4.0)
    .gradient(
      [1.0, 0.0, 0.0, 1.0], // Start color (red)
      [0.0, 0.0, 1.0, 1.0]  // End color (blue)
    )
    .line_cap(LineCap::Round)
    .render(gl)?;
  
  // Variable width line
  let points = vec![[0.0, 0.0], [0.25, 0.5], [0.5, 0.0], [0.75, 0.5], [1.0, 0.0]];
  let widths = vec![1.0, 3.0, 2.0, 4.0, 1.0];
  renderer.draw_line_variable_width(&points, &widths)
    .color([1.0, 1.0, 0.0, 1.0])
    .render(gl)?;
  
  Ok(())
}
```

## 📚 API Reference

### Core Types

| Type | Description | Use Case |
|------|-------------|----------|
| `LineRenderer` | Main rendering engine | All line drawing operations |
| `LineStyle` | Styling configuration | Colors, widths, patterns |
| `LineCap` | Line ending styles | Round, square, butt |
| `LineJoin` | Connection styles | Round, bevel, miter |

### Rendering Methods

| Method | Purpose | Example |
|--------|---------|---------|
| `draw_line()` | Render basic line | `renderer.draw_line(&points)` |
| `draw_curve()` | Render smooth curve | `renderer.draw_curve(&control_points)` |
| `draw_polyline()` | Connected line segments | `renderer.draw_polyline(&vertices)` |
| `draw_batch()` | Multiple lines efficiently | `renderer.draw_batch(&line_data)` |

### Styling Options

```rust,ignore
renderer.draw_line(&points)
  .width(2.5)                    // Line thickness
  .color([r, g, b, a])          // RGBA color
  .dash_pattern(&[dash, gap])    // Dash pattern
  .line_cap(LineCap::Round)      // Line endings
  .line_join(LineJoin::Miter)    // Connections
  .antialias(true)               // Smooth edges
  .render(gl)?;
```

## 🎯 Use Cases

- **Data Visualization** - Charts, graphs, and plotting applications
- **CAD Applications** - Technical drawing and design tools
- **Game Development** - UI elements, debug visualization, effects
- **Scientific Visualization** - Mathematical plots and simulations
- **Interactive Graphics** - Drawing applications and creative tools
- **Mapping Applications** - Route visualization and geographic data

## ⚡ Performance Tips

### Batch Rendering
```rust,ignore
// Efficient: batch multiple lines
let line1 = vec![[0.0, 0.0], [1.0, 0.0]];
let line2 = vec![[0.0, 1.0], [1.0, 1.0]];
let line3 = vec![[0.0, 2.0], [1.0, 2.0]];
let lines = vec![line1, line2, line3];
renderer.draw_batch(&lines).render(gl)?;

// Less efficient: individual calls
for line in lines {
  renderer.draw_line(&line).render(gl)?;
}
```

### Memory Management
- Reuse `LineRenderer` instances across frames
- Use batch rendering for large datasets
- Pre-allocate vertex buffers for dynamic content
- Cache styled line configurations

## 🔬 Advanced Features

### Matrix Transformations
Integration with ndarray_cg for complex transformations:

```rust,ignore
use ndarray_cg::*;
use std::f64::consts::PI;

let transform = mat3x3::scale([2.0, 1.5]) * mat3x3::rot(PI / 4.0);
let points = vec![[0.0, 0.0], [1.0, 1.0]];
renderer.set_transform(&transform);
renderer.draw_line(&points).render(gl)?;
```

### Custom Shaders
Extend functionality with custom GLSL shaders:

```rust,ignore
let vertex_shader_source = "#version 300 es\n...";
let fragment_shader_source = "#version 300 es\n...";
let custom_shader = renderer.create_custom_shader(
  vertex_shader_source,
  fragment_shader_source
)?;
renderer.use_shader(&custom_shader);
```

## 🛠️ Technical Details

### WebGL Implementation
- Uses WebGL 2.0 features for optimal performance
- Implements proper depth testing and blending
- Supports instanced rendering for batch operations
- Utilizes vertex buffer objects (VBOs) for efficiency

### Anti-Aliasing Techniques
- Multi-sample anti-aliasing (MSAA) support
- Screen-space anti-aliasing for thin lines
- Distance-based alpha blending for smooth edges
- Adaptive quality based on line width and zoom level

## 🚀 Getting Started

Add this to your `Cargo.toml`:

```toml
[dependencies]
line_tools = "0.1.0"
```

## 📖 Documentation

For detailed API documentation and examples, run:

```bash
cargo doc --open
```

## 🎮 Integration Examples

This crate works seamlessly with other cgtools modules:
- **minwebgl**: WebGL context management and shader utilities
- **ndarray_cg**: Mathematical transformations and matrix operations
- **browser_tools**: Logging and debugging in WebAssembly environments

Perfect for building comprehensive graphics applications with the cgtools ecosystem.
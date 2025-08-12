# tilemap_renderer

Agnostic 2D rendering engine with backend adapter support.

This crate provides a high-performance, backend-agnostic 2D rendering engine designed for maximum flexibility. The engine decouples scene definition from rendering implementation, allowing a single scene to be rendered across multiple backends using a Ports & Adapters architecture.

## features

- **Backend Agnostic**: Define scenes once, render anywhere
- **Ports & Adapters Architecture**: Clean separation between core logic and backend implementations  
- **High Performance**: Optimized for processing 10,000+ commands in under 16ms
- **Comprehensive Primitives**: Line, Curve, Text, Tilemap, and ParticleEmitter support
- **Scene Management**: Powerful command queue and querying system
- **Async Support**: Built for modern async/await workflows

## usage

This crate uses ultra-granular features for minimal builds. For full functionality, enable the `standard` feature:

```toml
[dependencies]
tilemap_renderer = { version = "0.1", features = ["standard"] }
```

```rust,ignore
// Example requires "standard" feature enabled
use tilemap_renderer::{ scene::Scene, commands::* };

// Create a new scene
let mut scene = Scene::new();

// Add rendering commands  
scene.add( RenderCommand::Line( LineCommand {
  start: Point2D { x: 0.0, y: 0.0 },
  end: Point2D { x: 100.0, y: 100.0 },
  style: StrokeStyle::default(),
}));

// Render with any backend adapter
// let mut renderer = SvgRenderer::new();
// renderer.render_scene( &scene )?;
```

## architecture

The crate is organized into several feature-gated modules:

- `scene` - Scene management and command queue system
- `commands` - All rendering primitives and command definitions  
- `ports` - Port traits for backend adapter integration
- `query` - Scene inspection and querying capabilities

## development status

This crate is currently in active development as part of the cgtools ecosystem. See `spec.md` and `roadmap.md` for detailed requirements and implementation progress.

## license

Licensed under MIT license.
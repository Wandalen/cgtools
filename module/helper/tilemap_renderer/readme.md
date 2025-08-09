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

```rust
// Basic usage example (implementation in progress)
// use tilemap_renderer::{ Scene, RenderCommand };
// 
// let mut scene = Scene::new();
// scene.add( RenderCommand::Line( /* ... */ ) );
// Render with any backend adapter
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
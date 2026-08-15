# shader_chunks_render_core

**Keywords:** WGSL, Headless Rendering, WebGPU, Offscreen, RGBA Readback

Renders a
[`shader_chunks_preview_core`](../shader_chunks_preview_core/readme.md)
`PreviewBundle` to raw RGBA pixels on a headless GPU — one static frame
of exactly what the
[`shader_chunks_preview_web`](../shader_chunks_preview_web/readme.md)
browser runner shows live, with every slider at its initial value and
`time` frozen at the caller's chosen instant. The whole graphics path is
[`minwgpu`](../../min/minwgpu/readme.md)'s offscreen toolkit: headless
context, one uniform buffer, the bufferless fullscreen-triangle pipeline,
a single clear-and-draw pass, and a row-padding-aware readback.

**Pipeline** (`render`):

```text
render( &bundle, ( width, height ), time )
  -> uniform_floats( bundle, size, time )      // time, params, pad, resolution
  -> minwgpu::context::headless()              // adapter + device + queue
  -> buffer/bind: one uniform at @group(0) @binding(0)
  -> texture::render_target_2d( Rgba8Unorm )   // NOT Srgb — see below
  -> pipeline::fullscreen + pass::draw_fullscreen   // vs_main / fs_main
  -> readback::rgba8                           // tightly packed RGBA8
  -> RenderedImage { pixels, size }
```

The uniform buffer follows the preview layout convention — `time : f32`
first, then each parameter as `f32` in declaration order, then
`resolution : vec4f` at the next 16-byte boundary — and [`uniform_floats`]
reuses
[`shader_chunks_preview_core::resolution_index`](../shader_chunks_preview_core/readme.md)
for that boundary, so this crate and the browser runner can never
disagree on the layout. The render target is `Rgba8Unorm`, deliberately
not `Rgba8UnormSrgb`: chunks write display-ready values (the collection's
`srgb` chunk exists precisely because encoding is the shader author's
explicit move), so an sRGB target would double-encode them.

No I/O and no image encoding happen here — the crate returns pixels;
writing a PNG is the CLI layer's job
([`shader_chunks_render`](../shader_chunks_render/readme.md)). The
bundle's WGSL is compiled by `wgpu` itself inside a validation error
scope, so even a broken shader fails loudly as [`RenderError::Gpu`],
never a panic — callers wanting friendlier diagnostics naga-validate
first, as `shader_chunks_render`'s `bundle_prepare` reuse does.

## Usage

```rust
use shader_chunks_preview_core::bundle_build;
use shader_chunks_render_core::render;

let chunk = shader_chunks_core::chunk_get( "fbm3" ).unwrap();
let bundle = bundle_build( chunk.wgsl ).unwrap();
let image = render( &bundle, ( 256, 256 ), 0.0 ).unwrap();
assert_eq!( image.size, ( 256, 256 ) );
assert_eq!( image.pixels.len(), 256 * 256 * 4 );
```

## Errors

[`RenderError`] is loud and ordered: `ZeroSize` fires before any GPU work,
`Context` means no usable headless adapter/device exists on the machine,
`Gpu` carries the device's validation error text (bad WGSL, a size beyond
the device's texture limit), and `Readback` covers the GPU→host copy.
Verify the exact-pixel contract yourself with
`cargo nextest run -p shader_chunks_render_core` — the constant-probe
test asserts every byte of a rendered frame.

# Hello Triangle Quickstart (WebGPU)

**Keywords:** WebGPU, Tutorial, Basics, Getting Started, Quickstart

Same triangle as [Hello Triangle](../hello_triangle/readme.md), built with minwebgpu's quickstart helpers instead of the raw step-by-step setup: `context::setup` collapses the from_canvas/request_adapter/request_device/preferred_format/configure sequence into one call, and `render_pass::draw_to` collapses the command encoder/render pass/submit ceremony into one call. Every value the helpers hand back — device, queue, format, the render pass itself — is still the plain native `web_sys` type, so dropping back to the manual API for any one step stays a normal function call away.

Compare this crate's `src/main.rs` against [Hello Triangle](../hello_triangle/readme.md)'s to see the raw API and the aggregated quickstart helpers side by side.

![image](./showcase.webp)

**[How to run](../../how_to_run.md)**

**References:**

* [WebGPU Specification]
* [WebGPU Fundamentals]

[WebGPU Specification]: https://www.w3.org/TR/webgpu/
[WebGPU Fundamentals]: https://webgpufundamentals.org

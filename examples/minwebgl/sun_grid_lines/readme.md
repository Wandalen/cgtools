# Sun Grid Lines

**Keywords:** WebGL2, Fragment Shader, Procedural Generation, Noise, Fullscreen Pass, Bloom, MRT, Post-Processing

![image](./showcase.webp)

A sci-fi HUD-style solar-system diagram, rendered by a fullscreen fragment shader — no meshes, no textures, no vertex buffer. A dark vertical-gradient background carries a noise-modulated nebula band and sparse twinkling stars; a grid overlays the whole scene; a granulated, noise-jagged star sits at the center inside a layered radial corona; a glowing ring traces its orbit, with one or more nodes circling it.

The vertex shader uses the "big triangle" trick (three vertices, no buffer, positions derived from `gl_VertexID`) to cover the viewport in a single draw call. All visual structure — gradient, grid, nebula fog, star field, corona, star-surface granulation, rim jitter, orbit ring, and orbiting nodes — is procedural, built from a compact hash-based value-noise/fbm function local to the shader (the workspace has no existing Perlin/Simplex implementation to depend on).

**Real multi-pass bloom.** The fragment shader writes two outputs via MRT: `frag_color` (the full composited scene) and `frag_emission` (only the layers that should glow — corona, star disk, ring, node halos; background/nebula/stars/grid are left black). Both land in an offscreen framebuffer. Every frame, `frag_emission` is fed through the `renderer` crate's real `UnrealBloomPass` (5-mip Gaussian blur + weighted composite), additively blended back onto `frag_color` via `BlendPass`, then presented to the screen through `ToSrgbPass`. This is the identical bloom → blend → present sequence the `renderer` crate's own `Renderer` uses internally, not a bespoke pipeline.

**Live parameterization.** Three uniforms drive the shader and can be changed at runtime with the keyboard:

| Key | Effect |
| :--- | :--- |
| `↑` / `↓` | Increase / decrease the number of orbiting nodes (1–8) |
| `←` / `→` | Decrease / increase grid density |
| `Space` | Reshuffle the seed — reshuffles the star field and every node's orbital phase/radius |

**[How to run](../how_to_run.md)**

**References:**

* [WebGL2 Fundamentals]
* [inigo quilez — fbm]

[WebGL2 Fundamentals]: https://webgl2fundamentals.org/
[inigo quilez — fbm]: https://iquilezles.org/articles/fbm/

# Sun Grid Lines

**Keywords:** WebGL2, Fragment Shader, Procedural Generation, Noise, Fullscreen Pass

A sci-fi HUD-style solar-system diagram, rendered entirely by a single fullscreen fragment shader — no meshes, no textures, no vertex buffer. A dark vertical-gradient background carries a noise-modulated nebula band and sparse twinkling stars; a translucent 10x10 grid overlays the whole scene; a granulated, noise-jagged star sits at the center inside a layered radial corona; a glowing ring traces its orbit, with one node slowly circling it.

The vertex shader uses the "big triangle" trick (three vertices, no buffer, positions derived from `gl_VertexID`) to cover the viewport in a single draw call. All visual structure — gradient, grid, nebula fog, star field, corona, star-surface granulation, rim jitter, orbit ring, and the orbiting node — is procedural, built from a compact hash-based value-noise/fbm function local to the shader (the workspace has no existing Perlin/Simplex implementation to depend on). Bloom is approximated analytically (radial falloff + `exp` glow terms) rather than via a separate multi-pass blur, keeping the example self-contained.

*(No showcase image bundled — this is a live animated WebGL canvas; run it locally to view, see How to run below.)*

**[How to run](../how_to_run.md)**

**References:**

* [WebGL2 Fundamentals]
* [inigo quilez — fbm]

[WebGL2 Fundamentals]: https://webgl2fundamentals.org/
[inigo quilez — fbm]: https://iquilezles.org/articles/fbm/

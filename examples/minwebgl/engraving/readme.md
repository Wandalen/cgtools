# Dynamic Text Engraving

**Keywords:** WebGL2, PBR, Engraving, Normal Mapping, glTF, Rust

This demo shows the renderer crate's dynamic text engraving pipeline running on a flat rectangular section of a cylinder's curved side. A JSON config (`engraving_config.json`) maps the cylinder's glTF node name to an engraving zone — its UV channel, aspect ratio, character limit and a whitelist of CSS fonts — and `EngravingSession` binds that config to the loaded scene, turning on the `PbrMaterial`'s `USE_ENGRAVING` shader path. Typing in the panel rasterizes white-on-black text to an offscreen canvas and uploads it to the GPU mask texture via `texSubImage2D`, with no shader recompilation.

On the shader side, `main.frag`'s `USE_ENGRAVING` block perturbs the surface normal from the mask's UV-space gradient (explicit-LOD `textureLod` central differences, matching the mip level implied by the fragment's screen-space UV footprint), approximating a beveled, laser-etched groove carved *into* the surface, and pushes roughness/albedo towards a matte, darkened look inside the mask without touching the base metal's hue. The GUI exposes the bevel strength, groove roughness and groove darkening as live sliders on top of the text/font controls, so the effect of each shader parameter can be inspected directly.

The cylinder asset (`assets/gltf/cylinder.glb`) ships two UV sets: `TEXCOORD_0` covers the whole curved surface for the base material, while `TEXCOORD_1` is a dedicated unwrap of a flat 1.96 (horizontal) x 2 (vertical) section on that curved side, reused directly as the engraving UV channel (`uvChannel: 1`, `aspectRatio: 0.98`) — no extra UV set needed to be authored for this demo.

**[How to run](../how_to_run.md)**

Preparing your own model (mesh topology, dual UV channels, export settings, `engraving_config.json` fields) for this pipeline: see the [artist guide](../../../module/helper/renderer/src/webgl/engraving/artist_guide.md).

**References:**

* [Normal Mapping Without Precomputed Tangents] - the derivative-based normal perturbation technique used by the engraving shader block
* [CSS Font Loading Module] - the `document.fonts.load()` API used to await web fonts before rasterizing

[Normal Mapping Without Precomputed Tangents]: http://www.thetenthplanet.de/archives/1180
[CSS Font Loading Module]: https://www.w3.org/TR/css-font-loading/

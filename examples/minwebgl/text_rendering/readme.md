# Text Rendering

**Keywords:** Text, Typography, WebGL2, UI

This demo demonstrates 3D text rendering in WebGL2. It uses UFO and TTF font glyphs for extracting its geometry data, then extrude them to create 3D text. Example demonstates several fonts rendering. This example uses simple layout technique for placing correctly words glyphs relatively to each other.

Below the font gallery, three more rows sweep the styling parameters that apply to any glyph mesh regardless of source font: **size** (`transform.scale`, 4 values), **color** (`PbrMaterial.base_color_factor`, 5 values), and **style modifiers** (bold, italic, underline, and all three combined). No bold/italic font face exists in this repository, so those two are synthesized on top of the regular glyph mesh: bold is a second, slightly-enlarged copy of the mesh drawn alongside the original; italic is a vertex-position shear. The underline is a measured quad sized to the string's own advance width.

![image](./showcase.webp)

**[How to run](../../how_to_run.md)**

**References:**

* [Unified Font Object]
* [TrueType]


[Unified Font Object]: https://unifiedfontobject.org/
[TrueType]: https://en.wikipedia.org/wiki/TrueType

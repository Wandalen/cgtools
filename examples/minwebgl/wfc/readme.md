# Wave Function Collapse

**Keywords:** Procedural Generation, WFC, Algorithm, WebGL2

This demo demonstrates the Wave Function Collapse algorithm for procedural content generation in WebGL2. WFC creates coherent patterns by propagating local constraints, useful for generating tile-based levels, textures, or structures.

WFC produces varied yet consistent results from small rule sets. This example shows practical implementation for game level generation or pattern synthesis.

![image](showcase.webp)

## Controls

- **File input**: Choose a Tiled `.tmx` map file from disk to use as the input
  pattern. The map's tile layer must use CSV encoding (in Tiled: Layer Format
  set to "CSV") -- other encodings (Base64, gzip/zlib) are not supported and
  are reported as an error instead of silently misreading the tile data.
- **Generate (wfc-image)**: Runs Wave Function Collapse using the currently
  loaded pattern (the uploaded TMX file, or the bundled default pattern if
  none was uploaded yet) and displays the generated result.

**[How to run](../../how_to_run.md)**

**References:**

* [WaveFunctionCollapse]
* [wfc (crate.io)]

[WaveFunctionCollapse]: https://github.com/mxgmn/WaveFunctionCollapse
[wfc (crate.io)]: https://crates.io/crates/wfc

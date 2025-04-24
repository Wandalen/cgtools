## Rust WFC libraries

In crates.io search request `wfc` gives such popular results:  

- ✅ - library compiles for WASM and returns consistent results.

- ⚠️ - library compiles for WASM, but not returns appropriate results.

- ❌ - library doesn't compiles for WASM and not returns appropriate results.


### ✅ wfc/wfc-image

**Source::** [link](https://crates.io/crates/wfc)

**Description:** **wfc** - library for generating grids of values which are similar to a specified grid. A typical use case for this is procedurally-generated images, though it generalizes to any grid of values.

Grids are populated using a constraint solver. For each cell, it is stored a probability distribution representing how likely that cell is to contain the top-left corner of possible pattern. Initially the probability of each pattern is based on its frequency in the sample image. Then, it repeatedly identifies the cell whose entropy is the lowest, and decides (randomly, weighted by probability distribution) which pattern to assign to the cell. This assignment may remove some candidate patterns from neighbouring cells, so it then updates candidate cells. This process of choosing a cell, assigning it a pattern, and propagating incompatible neighbours continues until either the entire grid is populated with values, or all the candidates are removed from a cell.

**wfc-image** - a helper for **wfc** to simplify generating images based on image files, using the image crate.

**Usability:** compiles for WASM. Uses simple API. Has example for usage. Returns consistent result event for big (100x100) tilemaps. Available setting: pattern size, output size, add pattern rotation, generation retry times, add formbidden patterns. 

### ❌ wfc-rs 

**Source::** [link](https://crates.io/crates/wfc-rs)

**Description:** The wfc-rs crate is a wrapper for the krychu/wfc implementation of Wave Function Collapse. The wfc library is manually wrapped with extern functions, and a small, more ideomatic Rust wrapper is provided.

**Usability:** this library uses libc that can't be compiled for WASM. Uses simple API. Has example for usage.

### ✅ wfc-tiled

**Source::** [link](https://crates.io/crates/wfc_tiled)

**Description:** This library contains helper functions to use the Wave Function Collapse algorithm provided by the wfc crate on tile-based maps.

You can load layer CSV files like the ones exported from Tiled, and save the result as another CSV or as a Tiled .tmx file for previewing inside the software.

As the underlying library only works on two dimensions, multiple layers are not supported.

**Usability:** This crate uses wfc. So it is analog of `wfc-image` library but with Tiled import/export integration.

### ❌ fastwfc

**Source::** [link](https://crates.io/crates/fastwfc)

**Description:** Rust bindings to libfastwfc. [fast-wfc](https://github.com/math-fehr/fast-wfc) is an implementation of Wave Function Collapse with a focus on performance. It was called fast-wfc because at the time it introduced optimizations improving the execution time by an order of magnitude.

**Usability:** this library uses libc that can't be compiled for WASM. Uses simple API. Documentation is incomplete.

### ⚠️ yawfc

**Source::** [link](https://crates.io/crates/yawfc)

**Description:** this is an unfinished pure rust implementation of the Wave Function Collapse Algorithm.

**Usability:** compiles for WASM. Has vulnerability to tiles conflicts. The generation cycle runs forever due to tiles conflicts. Uses simple API. Has example for usage.

Other libraries doesn't have complete, usable API or they are draft crates. 

## Libraries comparison

For comparison will be used: `wfc-image`, `wfc-rs`, `fastwfc`.

Table 1. Generation time (s) for different tile map size

| size \ lib | wfc-image | wfc-rs   | fastwfc |
|------------|-----------|----------|---------|
| 5x5        |  |  |  |
| 20x20      |  |  |  |
| 50x50      |  |  |  |
| 100x100    |  |  |  |
| 200x200    |  |  |  |

Table 2. Generation time (s) for tile map size 50x50 for different pattern size

| size \ lib | wfc-image | wfc-rs   | fastwfc |
|------------|-----------|----------|---------|
| 5x5        |  |  |  |
| 10x10      |  |  |  |
| 20x20      |  |  |  |
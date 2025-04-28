## Rust WFC libraries

In crates.io search request `wfc` gives such popular results:  

- ✅ - library compiles for WASM and returns consistent results.

- ⚠️ - library compiles for WASM, but not returns appropriate results.

- ❌ - library doesn't compiles for WASM and not returns appropriate results.


### ✅ wfc/wfc-image

**Source:** [wfc](https://crates.io/crates/wfc), [wfc-image](https://crates.io/crates/wfc_image)

**Description:** **wfc** - library for generating grids of values which are similar to a specified grid. A typical use case for this is procedurally-generated images, though it generalizes to any grid of values.

Grids are populated using a constraint solver. For each cell, it is stored a probability distribution representing how likely that cell is to contain the top-left corner of possible pattern. Initially the probability of each pattern is based on its frequency in the sample image. Then, it repeatedly identifies the cell whose entropy is the lowest, and decides (randomly, weighted by probability distribution) which pattern to assign to the cell. This assignment may remove some candidate patterns from neighbouring cells, so it then updates candidate cells. This process of choosing a cell, assigning it a pattern, and propagating incompatible neighbours continues until either the entire grid is populated with values, or all the candidates are removed from a cell.

**wfc-image** - a helper for **wfc** to simplify generating images based on image files, using the image crate.

**Features:**

- wfc, wfc-image - is libraries.

- Accepts as input a pattern image from which unique features will be separated.

- Pattern size determines the size of the unique features that will be extracted from the input pattern.

- You can specify the number of attempts if the algorithm fails to successfully complete the generation.

- For fine tuning features in generation results that can't be defined via input pattern, forbidden pattern can be used.

- This libraries have opportunity to set anchors around which the result will be formed. This feature gives a great opportunity to reproduce specific tile maps.

- This libraries have support for rotating and flipping input features or forbid rotation.

- There is a module for animating the generation process.

- Parallelism support.

- Overlap settings.

- Simple convertation: raw data -> input image -> output image -> raw data.

- The generation process can be reproduced with seed.

- Use image crate. That mean import of input and export of output as image are available. 

- Inspired by [Maxim Gumin](https://github.com/mxgmn/WaveFunctionCollapse), [Paul Merrell](https://paulmerrell.org/model-synthesis/), [Fehr Mathieu](https://github.com/math-fehr/fast-wfc) description and implementations of WFC algorithm.

**Usability:** compiles for WASM. Uses simple API. Has example for usage. Returns consistent result event for big (100x100 and more) tilemaps.  

### ❌ wfc-rs 

**Source:** [link](https://crates.io/crates/wfc-rs)

**Description:** The wfc-rs crate is a wrapper for the krychu/wfc implementation of Wave Function Collapse. The wfc library is manually wrapped with extern functions, and a small, more ideomatic Rust wrapper is provided.

**Features:**

- wfc-rs is wrapper lib of [krychu's wfc](https://github.com/krychu/wfc).

- Accepts as input a pattern image from which unique features will be separated.

- This library has support for rotating and flipping input features or forbid rotation.

- The generation process can be reproduced with seed.

- Simple convertation: raw data -> input image -> output image -> raw data

- Import of input and export of output as image are available. 

**Usability:** this library uses libc that can't be compiled for WASM. Uses simple API. Has example for usage.

### ✅ wfc-tiled

**Source:** [link](https://crates.io/crates/wfc_tiled)

**Description:** This library contains helper functions to use the Wave Function Collapse algorithm provided by the wfc crate on tile-based maps.

You can load layer CSV files like the ones exported from Tiled, and save the result as another CSV or as a Tiled .tmx file for previewing inside the software.

As the underlying library only works on two dimensions, multiple layers are not supported.

**Features:**

- wfc-tiled is library based on [wfc](https://crates.io/crates/wfc) crate. So features is similar to crate [wfc](https://crates.io/crates/wfc).

- Supports pattern import as CSV.

- Supports tile set import as image.

- Supports CSV, TMX(Tiled), image export.

**Usability:** This crate uses wfc. So it is analog of `wfc-image` library but with Tiled import/export integration.

### ❌ fastwfc

**Source:** [link](https://crates.io/crates/fastwfc)

**Description:** Rust bindings to libfastwfc. [fast-wfc](https://github.com/math-fehr/fast-wfc) is an implementation of Wave Function Collapse with a focus on performance. It was called fast-wfc because at the time it introduced optimizations improving the execution time by an order of magnitude.

**Features:**

- fastwfc is CLI and library wrapper of [fast-wfc](https://github.com/math-fehr/fast-wfc).

- Accepts as input a pattern image from which unique features will be separated.

- Pattern size determines the size of the unique features that will be extracted from the input pattern.

- Has tiles symmetry support.

- Has image pattern import and output export support with image crate.

**Usability:** this library uses libc that can't be compiled for WASM. Uses simple API. Documentation is incomplete.

### ⚠️ yawfc

**Source:** [link](https://crates.io/crates/yawfc)

**Description:** this is an unfinished pure rust implementation of the Wave Function Collapse Algorithm.

**Features:**

- yawfc is library.

- Accepts as input a matrix `Vec<Vec<T>>` from which unique features will be separated.

- For now pattern size of unique features is 3x3.

- This library has support for rotation and flipping of input features.

- Has debug mode.

- The generation process can be reproduced with seed.

- Then computing all possible ways the patterns can overlap without conflicting has complexity: O(n^2*N^2).

- Has image pattern import and output export support with image crate.

- Inspired by [mxgmn's](https://github.com/mxgmn/WaveFunctionCollapse) description and implementation.

**Usability:** compiles for WASM. Has vulnerability to tiles conflicts. The generation cycle runs forever due to tiles conflicts. Uses simple API. Has example for usage.

Other libraries doesn't have complete, usable API or they are draft crates. 

## Libraries comparison

### Features

| feature \ lib | wfc-image | wfc-rs    | wfc-tiled | fastwfc   | yawfc     |
|---------------|-----------|-----------|-----------|-----------|-----------|
| import |  |  |  |  |  |
| export |  |  |  |  |  |
| consistent results |  |  |  |  |  |
| platforms |  |  |  |  |  |
| pattern size |  |  |  |  |  |
| rotation |  |  |  |  |  |
| flipping |  |  |  |  |  |
| forbidden patterns |  |  |  |  |  |
| generate attempts |  |  |  |  |  |
| generation seed |  |  |  |  |  |
| docs |  |  |  |  |  |
| examples |  |  |  |  |  |
| CLI |  |  |  |  |  |
| is wrapper | no | yes | no | yes | no |
| animation | has | without | without | without | without |

### Speed

For comparison will be used: `wfc-image`, `wfc-rs`, `fastwfc`.

Table 2. Generation time (s) for different tile map size

| size \ lib | wfc-image | wfc-rs   | fastwfc |
|------------|-----------|----------|---------|
| 5x5        |  |  |  |
| 20x20      |  |  |  |
| 50x50      |  |  |  |
| 100x100    |  |  |  |
| 200x200    |  |  |  |

Table 3. Generation time (s) for tile map size 50x50 for different pattern size

| size \ lib | wfc-image | wfc-rs   | fastwfc |
|------------|-----------|----------|---------|
| 5x5        |  |  |  |
| 10x10      |  |  |  |
| 20x20      |  |  |  |
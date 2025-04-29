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

Table 1. Features support comparison for WFC libraries.

| feature \ lib      | wfc-image | wfc-rs    | wfc-tiled              | fastwfc   | yawfc     |
|--------------------|-----------|-----------|------------------------|-----------|-----------|
| import             | image     | image     | CSV                    | image     | image     |
| export             | image     | image     | CSV, TMX(Tiled), image | image     | image     |
| consistent results | ✅       | ✅        | ✅                    | ✅        | ❌        |
| WASM support       | ✅       | ❌        | ✅                    | ❌        | ✅        |
| pattern size       | ✅       | ❌        | ✅                    | ✅        | ❌        |
| rotation           | ✅       | ✅        | ✅                    | ❌        | ✅        |
| flipping           | ✅       | ✅        | ✅                    | ✅        | ✅        |
| forbidden patterns | ✅       | ❌        | ✅                    | ❌        | ❌        |
| generate attempts  | ✅       | ✅        | ✅                    | ✅        | ❌        |
| generation seed    | ✅       | ✅        | ✅                    | ❌        | ✅        |
| docs               | ✅       | ⚠️        | ✅                    | ❌        | ❌        |
| examples           | ✅       | ✅        | ✅                    | ❌        | ✅        |
| is wrapper         | ❌       | ✅        | ❌                    | ✅        | ❌        |
| animate generation | ✅       | ❌        | ❌                    | ❌        | ❌        |

### Work principles

**Explanation of features:**

**Pattern Models:** The `gridbugs/wfc` crate uses a more abstract "Pattern" concept defined by compatibility rules. `krychu/wfc` and `math-fehr/fast-wfc` adhere closely to the original WFC variants: Overlapping (patterns are square chunks of pixels) and Tiled (patterns are distinct tiles). `10maurycy10/wfc` focuses on the Tiled model.

**Collapsing Strategy (Observation):** All implementations listed likely use an entropy-based observation strategy or a scan for uncollapsed cells, which implicitly reduces entropy. `gridbugs/wfc` and `math-fehr/fast-wfc` explicitly use minimum entropy selection (with noise in `gridbugs`) via a priority queue or efficient scanning. The choice of which pattern to collapse to within the chosen cell is typically a weighted random selection based on pattern weights (if provided).

**Propagation:** The core of WFC is propagation. When a pattern becomes impossible for a cell, this new constraint affects its neighbors. All these implementations use some form of a propagation queue or stack. `gridbugs/wfc`'s `NumWaysToBecomePattern` is a specific internal mechanism to track local constraints and efficiently determine when a pattern becomes impossible for a cell due to a neighbor's state change. `fast-wfc` focuses on optimizing this propagation step using efficient data structures and precomputation.

**Constraint Models:** This refers to *how* compatibility is defined. In Overlapping WFC (`krychu`, `fast-wfc`), compatibility is implicit – two patterns are compatible in a direction if their overlapping parts match pixel-wise. In Tiled WFC (`gridbugs`, `krychu`, `fast-wfc`, `10maurycy10`), compatibility is explicit rules defined between pairs of tiles for each direction.

**Cell State:** How each cell's possibilities are stored varies. Using bitsets (`fast-wfc` and likely others) is common for performance, as bitwise operations can quickly check pattern validity or update possibilities. `gridbugs/wfc` adds the `NumWaysToBecomePattern` layer to track *why* a pattern is possible (based on neighbor counts), which aids its specific propagation mechanism.

**Table 2.** Work principles comparison for WFC libraries.

| Feature (Work Principle)         | `gridbugs/wfc`                                                                | `krychu/wfc`                                                                     | `math-fehr/fast-wfc`                                                                 | `10maurycy10/wfc`                                                                |
| :------------------------------- | :---------------------------------------------------------------------------- | :------------------------------------------------------------------------------- | :----------------------------------------------------------------------------------- | :------------------------------------------------------------------------------- |
| **Algorithm Variant**            | Entropy-based (minimizes uncertainty)                                         | Implements both Simple Tiled and Overlapping WFC.                              | Entropy-based (port of a C++ implementation known for performance). Supports Tiled and Overlapping. | Simple Tiled WFC.                                                                |
| **Pattern Model / Constraint Definition** | Generic Tiled/Pattern model. Patterns are defined explicitly with detailed neighbor compatibility rules per direction (`CardinalDirectionTable<Vec<PatternId>>`). Supports optional weights. | Overlapping: Extracts patterns from input image, compatibility based on pixel overlap. Tiled: Explicit tile definitions and compatibility rules. | Supports both Overlapping and Tiled models. Compatibility rules derived from input (Overlapping) or explicitly defined (Tiled). | Tiled model. Explicit tile definitions and compatibility rules.                  |
| **Cell State Representation**    | Each cell (`WaveCell`) tracks its entropy stats and `NumWaysToBecomePattern` for each possible pattern, indicating how many ways each pattern is still supported by neighbors in each direction. | Typically uses data structures like bitsets or vectors to track the set of currently possible patterns for each cell. | Often uses optimized data structures like bitsets for efficient tracking of possible patterns per cell. | Likely uses data structures (e.g., vectors or bitsets) to track possible patterns per cell. |
| **Observation Strategy (Collapsing)** | Selects the cell with the minimum entropy (using a priority queue) to collapse. Uses a random value (`noise`) for tie-breaking. Chooses a pattern from the compatible set based on weights. | Scans for an uncollapsed cell, likely prioritizing based on minimum entropy, then performs a weighted random selection among possible patterns. | Efficiently scans for the minimum entropy cell. Performs weighted random selection among possible patterns. | Likely scans for an uncollapsed cell, potentially using minimum entropy as a heuristic, then performs random (possibly weighted) selection. |
| **Propagation Mechanism**        | Queue-based propagation (`Propagator`). When a pattern is removed from a cell, the change is queued and processed iteratively. `NumWaysToBecomePattern` helps efficiently update neighbor constraints. | Standard queue-based propagation. When a pattern is ruled out for a cell, this constraint change is added to a queue/stack to update adjacent cells. | Optimized queue-based propagation. Performance relies on efficient constraint checking using precomputed data and bitwise operations. | Queue-based propagation.                                                         |
| **Contradiction Handling**       | Detects contradictions if a cell's possibility count drops to zero. Provides an interface (`collapse_retrying`) for retrying the collapse from scratch upon contradiction. | Detects contradictions. Might offer retry mechanisms or require manual handling. | Detects contradictions. Often includes built-in retry or allows external handling. | Detects contradictions. Handling (e.g., retry) depends on the API.                |
| **Boundary Conditions**          | Supports explicit wrapping via the `Wrap` trait (e.g., toroidal).             | Likely supports various boundary conditions (periodic, fixed/blank edges) common in WFC libs. | Likely supports various boundary conditions (periodic, fixed).                     | Support depends on implementation details, likely includes periodic or fixed.    |
| **Implementation Focus**         | Flexible API, custom pattern definitions, explicit control over state and propagation. | Implements standard Overlapping and Tiled WFC algorithms as described in literature. | High performance and speed, porting optimizations from the C++ library.           | Straightforward implementation of Tiled WFC.                                     |

### Supported tile sets

All WFC libs get as input pattern. Input pattern are splitted on unique parts of constant size. This parts contain relations information between tiles. Therefore generation output depends on input pattern tiles adjacency. If tiles adjacency in input pattern is complete (doesn't have contradictions) then WFC lib can support generation for every type of square grid tile sets from [site](https://www.boristhebrave.com/permanent/24/06/cr31/stagecast/wang/blob.html).

All WFC libs have neighbors constraint according to [site](https://www.redblobgames.com/grids/parts/#square-relationships).
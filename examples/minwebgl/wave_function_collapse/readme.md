# 🌊 Wave Function Collapse

**Procedural Tile Map Generation with WebGL Rendering**

A comprehensive demonstration of the Wave Function Collapse (WFC) algorithm for procedural tile map generation, rendered in real-time using WebGL2. This example showcases how to create coherent, rule-based tile patterns that can be used for game worlds, level generation, and procedural content creation.

<p align="center">
  <img src="./showcase.png" width="600px">
</p>

## ✨ Features

- **🔄 Wave Function Collapse Algorithm**: Complete WFC implementation with constraint propagation
- **🎲 Procedural Generation**: Generate infinite variations of tile-based content  
- **⚡ WebGL2 Rendering**: Hardware-accelerated tile rendering with texture atlases
- **🧩 Configurable Rules**: Customizable adjacency relations between tiles
- **🎨 Custom Tilesets**: Support for user-provided tile textures
- **🔧 Real-time Generation**: Interactive map generation with immediate visual feedback

Rendering details description can be found in [ mapgen_tiles_rendering ]( ../mapgen_tiles_rendering/readme.md ) example

## 🎯 What You'll Learn

This example demonstrates:
- **Algorithm Implementation**: How WFC algorithm works step-by-step
- **Constraint Solving**: Managing tile adjacency rules and propagation
- **Texture Management**: Converting tile data to GPU textures efficiently
- **Real-time Generation**: Interactive procedural content creation

For detailed rendering implementation, see the [Tile Map Rendering](../mapgen_tiles_rendering/README.md) example.

## 🔬 How It Works

The Wave Function Collapse algorithm follows these key steps:

### 1. 🎯 **Initialization**
- Configure map size, tile relations, and initial constraints
- Set up the "superposition" state where each cell can be any tile

### 2. 🔄 **Main Algorithm Loop**
The algorithm repeats the **collapse-propagate** cycle until completion:

#### **Collapse Phase** 🎲
- Find cells with minimum entropy (fewest possible states)
- Randomly choose one valid tile for that cell
- "Collapse" the quantum superposition to a definite state

#### **Propagate Phase** 📡  
- Update neighboring cells based on adjacency rules
- Remove invalid possibilities from adjacent cells
- Propagate constraints throughout the grid

### 3. ✅ **Completion**
- Continue until all cells are resolved or contradiction detected
- Handle any remaining conflicts or impossible states
- Return the final coherent tile map

### Running

Make sure you have installed all the necessary [ dependencies ]( ../../../module/min/minwebgl/readme.md )
In order to run the example navigate to example's directory and run next command:
``` bash
trunk serve
```
If you want to load own tile set image, upload it into `resources` folder as `tileset.png` and set tile count in const LAYERS. 

``` rust
const LAYERS : i32 = 6;
```

Tileset image must store tile textures from up to down order, all textures must have equal size without align.

You can change tile map size with SIZE const. If width don't be equal height than tiles will be rectangles not squares stretched along quad. 

``` rust
const SIZE : ( usize, usize ) = ( 32, 32 );
```

If you want set own adjacent relations between tiles, you can change RELATIONS const. List depends from tile variants count. Each items id in RELATIONS equals to tile id. Each item contains list of possible adjacent tiles for current tile id. Changing RELATIONS, LAYERS, SIZE, `tileset.png` texture may create new patterns. 

``` rust
const RELATIONS : &str = "
  [
    [ 0, 1 ],
    [ 1, 2 ],
    [ 2, 3 ],
    [ 3, 4 ],
    [ 4, 5 ],
    [ 5 ]
  ]
";
```
# 🗺️ WebGL Hexagonal Map Editor

> **A high-performance, interactive hexagonal map editor built with Rust, WebAssembly, and WebGL.**

This demo showcases a versatile and efficient engine for rendering and editing large-scale hexagonal tile maps directly in the browser. It features a robust toolset for world-building, including terrain/object placement, player ownership, and river drawing. The entire map state can be easily saved and loaded via JSON.

![Hexagonal Map Editor](image.png)

## ✨ Features

### 🗺️ **Interactive Map Editing**

- **Dynamic Camera** - Smooth panning (Space + Drag) and zooming (Mouse Wheel).
- **Dual Edit Modes** - Seamlessly switch between editing tiles and drawing rivers.
- **Tile Painting** - Place various objects (buildings, foliage, units) and assign tile ownership to players.
- **River System** - Intuitively draw rivers by connecting vertices on the hex grid.

### ⚙️ **Efficient Rendering Engine**

- **GPU Accelerated** - Leverages WebGL to render vast maps with thousands of tiles at high frame rates.
- **Instanced Rendering** - A single hexagon mesh is drawn once and then instanced for every tile, minimizing draw calls.
- **Sprite Support** - Renders PNG sprites for map objects, loaded from a configuration file.

### 💾 **Data-Driven & Portable**

- **JSON Configuration** - Map objects, sprites, and player colors are defined in an external `config.json`, making the editor easily customizable.
- **Save & Load** - Download the complete map state as a JSON file or load one via drag-and-drop.
- **Rust + Wasm** - The performance and safety of Rust compiled to a portable and fast WebAssembly module.

## 🚀 Quick Start

### Prerequisites

- WebGL 2.0 compatible browser
- Rust with `wasm32-unknown-unknown` target
- Trunk for development server

### Run the Example

```bash
# Navigate to the hexagonal map example
cd examples/minwebgl/hexagonal_map

# Install trunk if needed
cargo install trunk

# Serve the example
trunk serve --release
```

Open `http://localhost:8080` to start building your world.

## 🔧 Technical Deep Dive

### Hexagonal Grid Logic

The map is built on a hexagonal grid using **Axial Coordinates** (`q`, `r`). This system simplifies many common grid operations like distance calculation, neighbor finding, and pathfinding. Mouse clicks in pixel space are converted into the corresponding hexagonal coordinate to identify the tile being edited.

For river placement, a **Tri-Axial coordinate** system is used to uniquely identify the vertices where three hexagons meet, allowing for a precise and flexible river drawing system.

## 🦀 Rust Implementation

The core logic is written in Rust, defining clear data structures for the map state which can be easily serialized to and from JSON.

### Core Data Structures (`src/core_game.rs`)

```rust
// Represents a single hexagonal tile on the map
#[ derive( Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq ) ]
pub struct Tile
{
  pub object_index : Option< ObjectIndex >,
  pub terrain_index : TerraintIndex,
  pub owner_index : PlayerIndex,
  pub coord : Coord, // Axial coordinate (q, r)
}

// The main map structure holding all tiles and rivers
#[ serde_as ]
#[derive( Debug, Clone, Serialize, Deserialize, Default ) ]
pub struct Map
{
  #[ serde_as( as = "Vec< ( _, _ ) >" ) ]
  pub tiles : FxHashMap< Coord, Tile >,
  pub rivers : FxHashSet< [ triaxial::TriAxial; 2 ] >,
}

// Defines a configurable object property, like a castle or tree
#[derive( Debug, Serialize, Deserialize ) ]
pub struct Properties
{
  pub name : String,
  pub attributes : serde_json::Map< String, serde_json::Value >,
  pub sprite : Option< Sprite >,
}
```

### Main Loop and Input Handling (`src/main.rs`)

The main loop continuously checks for user input, updates the map state, and then issues rendering commands.

```rust
// A snippet from the main update loop
fn update()
{
  input.update_state();

  // ... handle camera zoom and pan from mouse/keyboard ...

  // Convert mouse screen position to world/grid coordinates
  let pointer_pos = screen_to_world( input.pointer_position(), ... );
  let pixel : Pixel = ( pointer_pos - camera_pos ).into();
  let hexagon_coord: Coord = pixel.into();

  // Handle editing based on selected mode
  if edit_mode == EditMode::EditTiles
  {
    if input.is_button_down(MouseButton::Main)
    {
      // Create and insert a new tile into the map
      let tile = core_game::Tile { ... };
      map.borrow_mut().tiles.insert( hexagon_coord, tile );
    }
    else if input.is_button_down( MouseButton::Secondary )
    {
      // Remove the tile
      map.borrow_mut().tiles.remove( &hexagon_coord );
    }
  }
  else if edit_mode == EditMode::EditRivers
  {
    // ... logic for adding/removing river segments ...
  }

  // ... prepare and execute rendering commands ...
}
```

## 🎮 Use Cases and Applications

### Game Development

- **Level Design** - A powerful tool for designing levels for strategy games, 4X games, or turn-based tactical RPGs.
- **Prototyping** - Quickly prototype game mechanics on a hex grid.
- **In-Game Editor** - The engine could be integrated into a game to allow for user-generated maps.

### World-Building & Simulation

- **Tabletop RPGs** - Create and share detailed world maps for campaigns.
- **Simulations** - Visualize geographic or abstract data on a hexagonal grid, useful for simulating population spread, resource distribution, and more.

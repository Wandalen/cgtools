# 🎮 Conway's Game of Life in Rust

> **Cellular automaton simulation using efficient ndarray operations**

A comprehensive implementation of **Conway's Game of Life** using the `ndarray_cg` crate for high-performance 2D array manipulation. This mathematical simulation demonstrates emergent complexity from simple rules, showcasing pattern evolution, cellular automata theory, and efficient grid computation techniques.

![Game of Life Animation](./showcase.gif)

## ✨ Features

### 🧬 **Cellular Automata**
- **Conway's Rules** - Classic Game of Life rule implementation
- **Grid Evolution** - Step-by-step cellular state transitions
- **Pattern Recognition** - Identify stable, oscillating, and moving patterns
- **Emergent Behavior** - Complex patterns from simple rules

### 🔧 **Technical Implementation**
- **ndarray Optimization** - Vectorized operations for performance
- **Memory Efficient** - Optimized grid storage and manipulation
- **Configurable Simulation** - Adjustable grid size and step count
- **File-Based Initialization** - Load custom starting patterns

### 📊 **Educational Value**
- **Mathematical Concepts** - Discrete mathematics and cellular automata
- **Algorithm Design** - Efficient neighbor counting and rule application
- **Pattern Analysis** - Study of emergent complexity in simple systems
- **Performance Optimization** - Array processing techniques

## 🚀 Quick Start

### Prerequisites
- Rust with Cargo
- Text editor for creating initial patterns

### Run the Simulation
```bash
# Navigate to Game of Life example
cd examples/math/life

# Run with default configuration
cargo run

# Run with custom steps
cargo run -- --steps 500
```

## 🎲 Game Rules

Conway's Game of Life follows four fundamental rules:

### 🌱 **Birth Rule**
- A **dead cell** with exactly **3 live neighbors** becomes alive
- Simulates reproduction when conditions are optimal

### ❤️ **Survival Rule** 
- A **live cell** with **2 or 3 live neighbors** survives
- Represents stable population with adequate resources

### 💀 **Death Rules**
- **Underpopulation**: Live cell with <2 neighbors dies (loneliness)
- **Overpopulation**: Live cell with >3 neighbors dies (overcrowding)

```rust
// Rule implementation
fn apply_life_rules(current: bool, live_neighbors: usize) -> bool {
  match (current, live_neighbors) {
    (true, 2) | (true, 3) => true,  // Survival
    (false, 3) => true,             // Birth
    _ => false,                     // Death
  }
}
```

## 🔧 Technical Deep Dive

### Efficient Grid Processing

The implementation uses `ndarray_cg` for vectorized operations:

```rust
use ndarray_cg::Array2D;

// Grid representation
struct LifeGrid {
  current: Array2D<bool>,
  next: Array2D<bool>,
  width: usize,
  height: usize,
}

impl LifeGrid {
  fn new(width: usize, height: usize) -> Self {
    Self {
      current: Array2D::zeros((height, width)),
      next: Array2D::zeros((height, width)),
      width,
      height,
    }
  }
  
  // Efficient neighbor counting
  fn count_neighbors(&self, row: usize, col: usize) -> usize {
    let mut count = 0;
    
    for dr in -1..=1 {
      for dc in -1..=1 {
        if dr == 0 && dc == 0 { continue; } // Skip self
        
        let r = (row as isize + dr) as usize;
        let c = (col as isize + dc) as usize;
        
        if r < self.height && c < self.width {
          if self.current[[r, c]] {
            count += 1;
          }
        }
      }
    }
    
    count
  }
}
```

### Optimized Evolution Step

```rust
impl LifeGrid {
  fn evolve(&mut self) {
    // Process all cells in parallel-friendly manner
    for row in 0..self.height {
      for col in 0..self.width {
        let neighbors = self.count_neighbors(row, col);
        let current_state = self.current[[row, col]];
        
        self.next[[row, col]] = match (current_state, neighbors) {
          (true, 2) | (true, 3) => true,  // Survival
          (false, 3) => true,             // Birth  
          _ => false,                     // Death
        };
      }
    }
    
    // Swap buffers for next iteration
    std::mem::swap(&mut self.current, &mut self.next);
  }
  
  fn simulate(&mut self, steps: usize) {
    for step in 0..steps {
      self.evolve();
      
      if step % 10 == 0 {
        println!("Step {}: {} live cells", step, self.count_live_cells());
      }
    }
  }
}
```

### Pattern Loading System

```rust
// Load patterns from text files
fn load_pattern_from_file(filename: &str) -> Result<LifeGrid, std::io::Error> {
  let content = std::fs::read_to_string(filename)?;
  let lines: Vec<&str> = content.lines().collect();
  
  let height = lines.len();
  let width = lines.iter().map(|line| line.len()).max().unwrap_or(0);
  
  let mut grid = LifeGrid::new(width, height);
  
  for (row, line) in lines.iter().enumerate() {
    for (col, ch) in line.chars().enumerate() {
      grid.current[[row, col]] = ch == '#';
    }
  }
  
  Ok(grid)
}
```

## 📝 Pattern Format

### Input File Format (`life.txt`)

The input file uses a simple text format:
- `#` = Live cell (populated)
- `.` = Dead cell (empty)
- Any other character = Dead cell

```
# Example pattern: Glider
.#.
..#
###
```

### Famous Patterns

#### Glider (Moving Pattern)
```
.#.
..#
###
```

#### Oscillator (Blinker)
```
###
```

#### Still Life (Block)
```
##
##
```

#### Spaceship
```
.#..#
....#
.####
```

## 🎯 Pattern Analysis

### Pattern Categories

```rust
// Pattern classification
enum PatternType {
  StillLife,    // No change between generations
  Oscillator,   // Cycles through states
  Spaceship,    // Moving patterns
  Methuselah,   // Long-lived chaotic patterns
  Garden,       // Dense random patterns
}

struct PatternAnalyzer {
  history: Vec<Array2D<bool>>,
  cycle_detector: CycleDetector,
}

impl PatternAnalyzer {
  fn analyze_pattern(&mut self, grid: &LifeGrid) -> PatternType {
    self.history.push(grid.current.clone());
    
    // Check for still life
    if self.is_still_life() {
      return PatternType::StillLife;
    }
    
    // Check for oscillation
    if let Some(period) = self.detect_oscillation() {
      return PatternType::Oscillator;
    }
    
    // Analyze movement patterns
    if self.detect_translation() {
      return PatternType::Spaceship;
    }
    
    PatternType::Garden // Default for complex patterns
  }
}
```

### Statistical Analysis

```rust
// Gather simulation statistics
struct LifeStatistics {
  generation: usize,
  live_cells: usize,
  birth_count: usize,
  death_count: usize,
  stability_measure: f64,
}

impl LifeGrid {
  fn gather_statistics(&self, generation: usize) -> LifeStatistics {
    let live_cells = self.count_live_cells();
    let density = live_cells as f64 / (self.width * self.height) as f64;
    
    LifeStatistics {
      generation,
      live_cells,
      birth_count: self.count_births(),
      death_count: self.count_deaths(),
      stability_measure: self.calculate_stability(),
    }
  }
  
  fn calculate_stability(&self) -> f64 {
    // Measure how much the pattern changes between generations
    let changes = self.count_cell_changes();
    1.0 - (changes as f64 / (self.width * self.height) as f64)
  }
}
```

## ⚙️ Configuration Options

### Grid Customization

```rust
// Configurable parameters
struct SimulationConfig {
  grid_width: usize,
  grid_height: usize, 
  max_generations: usize,
  initial_pattern: PatternSource,
  boundary_conditions: BoundaryType,
  output_frequency: usize,
}

enum BoundaryType {
  Dead,      // Cells outside grid are always dead
  Toroidal,  // Grid wraps around (torus topology)
  Reflected, // Boundaries reflect back
}

enum PatternSource {
  File(String),
  Random(f64), // Random with specified density
  Preset(PresetPattern),
}
```

### Performance Optimization

```rust
// Optimized implementations for large grids
struct OptimizedLifeGrid {
  sparse_representation: HashMap<(usize, usize), bool>,
  bounding_box: Rect,
  generation: usize,
}

impl OptimizedLifeGrid {
  // Only process active regions
  fn evolve_sparse(&mut self) {
    let mut changes = Vec::new();
    
    // Expand bounding box to include potential births
    let expanded_box = self.bounding_box.expand(1);
    
    for row in expanded_box.top..=expanded_box.bottom {
      for col in expanded_box.left..=expanded_box.right {
        let neighbors = self.count_neighbors_sparse(row, col);
        let current = self.get_cell(row, col);
        let next = apply_life_rules(current, neighbors);
        
        if current != next {
          changes.push(((row, col), next));
        }
      }
    }
    
    // Apply all changes
    for ((row, col), state) in changes {
      if state {
        self.sparse_representation.insert((row, col), true);
      } else {
        self.sparse_representation.remove(&(row, col));
      }
    }
    
    self.update_bounding_box();
    self.generation += 1;
  }
}
```

## 📊 Output Analysis

The simulation produces detailed evolution tracking:

```
# Generation 0
...........................
.###.......#####........##.
.#..#.....#....##.###...##.
..#.#.....#........##......
...#......#.....#.###......
...........#.##............

# Generation 1
.#.#..........##........##.
.###.......######....#.##..
..##.....##....###..##.....
..........#.....#.##.......
...........###.............  
...........###.............

# Statistics
Generation: 50
Live Cells: 127
Births this step: 23
Deaths this step: 19
Stability: 0.85
```

## 🎨 Visualization Extensions

### ASCII Art Rendering

```rust
// Enhanced ASCII visualization
struct ASCIIRenderer {
  live_char: char,
  dead_char: char,
  border_style: BorderStyle,
}

impl ASCIIRenderer {
  fn render_with_stats(&self, grid: &LifeGrid, stats: &LifeStatistics) {
    println!("┌── Generation {} ──┐", stats.generation);
    
    for row in 0..grid.height {
      print!("│ ");
      for col in 0..grid.width {
        let ch = if grid.current[[row, col]] {
          self.live_char
        } else {
          self.dead_char
        };
        print!("{}", ch);
      }
      println!(" │");
    }
    
    println!("└── Live: {} ──┘", stats.live_cells);
  }
}
```

## 📚 Learning Resources

### Cellular Automata Theory
- **[Cellular Automata](https://en.wikipedia.org/wiki/Cellular_automaton)** - Mathematical foundation
- **[Game of Life Patterns](https://conwaylife.com/)** - Comprehensive pattern database
- **[Emergent Behavior](https://en.wikipedia.org/wiki/Emergence)** - Complex systems theory

### Implementation Techniques
- **[ndarray Documentation](https://docs.rs/ndarray/latest/ndarray/)** - Array processing in Rust
- **[Algorithm Optimization](https://en.wikipedia.org/wiki/Hashlife)** - Advanced Life algorithms
- **[Sparse Arrays](https://en.wikipedia.org/wiki/Sparse_array)** - Memory-efficient representations

## 🛠️ Advanced Features

### Multi-Threaded Processing

```rust
use rayon::prelude::*;

impl LifeGrid {
  fn evolve_parallel(&mut self) {
    let next: Vec<Vec<bool>> = (0..self.height)
      .into_par_iter()
      .map(|row| {
        (0..self.width)
          .map(|col| {
            let neighbors = self.count_neighbors(row, col);
            let current = self.current[[row, col]];
            apply_life_rules(current, neighbors)
          })
          .collect()
      })
      .collect();
    
    // Copy results back to grid
    for (row, row_data) in next.into_iter().enumerate() {
      for (col, cell_state) in row_data.into_iter().enumerate() {
        self.next[[row, col]] = cell_state;
      }
    }
    
    std::mem::swap(&mut self.current, &mut self.next);
  }
}
```

## 🤝 Contributing

Part of the CGTools workspace. Feel free to submit issues and pull requests on GitHub.

## 📄 License

MIT

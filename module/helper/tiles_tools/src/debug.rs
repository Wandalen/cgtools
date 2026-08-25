
//! Visual debugging tools and utilities for tile-based game development.
//!
//! This module provides comprehensive debugging capabilities including grid visualization,
//! pathfinding overlays, ECS component inspection, performance profiling, and diagnostic
//! tools. These utilities are essential for development, testing, and optimization of
//! tile-based games.
//!
//! # Debugging Features
//!
//! - **Grid Visualization**: Render coordinate systems with customizable styles
//! - **Pathfinding Debug**: Visualize A* paths, flow fields, and navigation costs
//! - **ECS Inspector**: Runtime component inspection and entity tracking
//! - **Performance Profiler**: Frame timing, memory usage, and bottleneck detection
//! - **Spatial Debug**: Quadtree visualization and collision boundary display
//! - **Event Monitoring**: Real-time event system diagnostics
//!
//! # Debug Output Formats
//!
//! - **ASCII Art**: Console-based visualization for headless debugging
//! - **SVG Export**: Vector graphics for documentation and analysis
//! - **JSON Reports**: Machine-readable diagnostic data
//! - **HTML Dashboard**: Interactive web-based debugging interface
//!
//! # Examples
//!
//! ```rust
//! use tiles_tools::debug::*;
//! use tiles_tools::coordinates::hexagonal::{Coordinate, Axial, Pointy};
//!
//! // Create a debug grid renderer
//! let mut renderer = GridRenderer::new()
//!     .with_size(10, 8)
//!     .with_style(GridStyle::Hexagonal);
//!
//! // Add some debug markers
//! renderer.marker_add(Coordinate::<Axial, Pointy>::new(2, 3), "S", "Start position");
//! renderer.marker_add(Coordinate::<Axial, Pointy>::new(7, 5), "G", "Goal position");
//!
//! // Render as ASCII art
//! println!("{}", renderer.ascii_render());
//!
//! // Export as SVG
//! renderer.svg_export("-debug_grid.svg").expect("Failed to export SVG");
//! ```

use std::collections::{HashMap, VecDeque};
use std::time::{Instant, Duration};
use std::fs::{File, create_dir_all};
use std::io::{Write, BufWriter};
use std::fmt::Write as _;
use std::path::Path;

/// Visual debugging renderer for coordinate grids.
pub struct GridRenderer
{
  width: usize,
  height: usize,
  style: GridStyle,
  markers: HashMap<(i32, i32), DebugMarker>,
  highlights: Vec<DebugHighlight>,
  annotations: Vec<DebugAnnotation>,
}

/// Style options for grid rendering.
#[ derive( Debug, Clone, Copy ) ]
pub enum GridStyle
{
  /// Square grid with 4-connected neighbors
  Square4,
  /// Square grid with 8-connected neighbors
  Square8,
  /// Hexagonal grid with pointy-top orientation
  Hexagonal,
  /// Triangular tessellation
  Triangular,
  /// Isometric projection
  Isometric,
}

/// Debug marker for highlighting specific coordinates.
#[ derive( Debug, Clone ) ]
pub struct DebugMarker
{
  /// Display symbol (single character)
  pub symbol: String,
  /// Tooltip description
  pub description: String,
  /// Display color (for colored output)
  pub color: DebugColor,
  /// Marker priority (higher priority shown on top)
  pub priority: u32,
}

/// Debug highlight for marking areas or paths.
#[derive(Debug, Clone)]
pub struct DebugHighlight {
  /// Coordinates to highlight
  pub coordinates: Vec<(i32, i32)>,
  /// Highlight style
  pub style: HighlightStyle,
  /// Color for the highlight
  pub color: DebugColor,
  /// Description of the highlight
  pub label: String,
}

/// Styles for highlighting areas.
#[derive(Debug, Clone, Copy)]
pub enum HighlightStyle {
  /// Outline the highlighted area
  Outline,
  /// Fill the highlighted area
  Fill,
  /// Show as dotted lines
  Dotted,
  /// Animated highlight (for interactive displays)
  Animated,
}

/// Debug annotation for adding text labels.
#[derive(Debug, Clone)]
pub struct DebugAnnotation {
  /// Position of the annotation
  pub position: (i32, i32),
  /// Text content
  pub text: String,
  /// Text color
  pub color: DebugColor,
  /// Offset from the coordinate center
  pub offset: (i32, i32),
}

/// Color options for debug rendering.
#[derive(Debug, Clone, Copy)]
pub enum DebugColor {
  /// Default color (usually white/black)
  Default,
  /// Red color (errors, obstacles)
  Red,
  /// Green color (valid paths, goals)
  Green,
  /// Blue color (water, special areas)
  Blue,
  /// Yellow color (warnings, temporary)
  Yellow,
  /// Purple color (special entities)
  Purple,
  /// Orange color (intermediate states)
  Orange,
  /// Gray color (disabled/inactive)
  Gray,
}

impl GridRenderer
{
  /// Creates a new grid renderer.
  #[must_use]
  pub fn new() -> Self
  {
    Self
    {
      width: 20,
      height: 15,
      style: GridStyle::Square4,
      markers: HashMap::new(),
      highlights: Vec::new(),
      annotations: Vec::new(),
    }
  }

  /// Sets the grid size.
  #[must_use]
  pub fn with_size(mut self, width: usize, height: usize) -> Self {
    self.width = width;
    self.height = height;
    self
  }

  /// Sets the grid style.
  #[must_use]
  pub fn with_style(mut self, style: GridStyle) -> Self {
    self.style = style;
    self
  }

  /// Returns the configured grid width.
  #[ must_use ]
  pub fn width( &self ) -> usize
  {
    self.width
  }

  /// Returns the configured grid height.
  #[ must_use ]
  pub fn height( &self ) -> usize
  {
    self.height
  }

  /// Returns the configured grid style.
  #[ must_use ]
  pub fn style( &self ) -> GridStyle
  {
    self.style
  }

  /// Returns how many markers are currently stored.
  #[ must_use ]
  pub fn marker_count( &self ) -> usize
  {
    self.markers.len()
  }

  /// Returns `true` when a marker is stored at `pos`.
  #[ must_use ]
  pub fn has_marker( &self, pos : ( i32, i32 ) ) -> bool
  {
    self.markers.contains_key( &pos )
  }

  /// Adds a debug marker at the specified coordinate.
  pub fn marker_add<C>(&mut self, coord: C, symbol: &str, description: &str)
  where
    C: Into<(i32, i32)>,
  {
    let pos = coord.into();
    self.markers.insert(pos, DebugMarker {
      symbol: symbol.to_string(),
      description: description.to_string(),
      color: DebugColor::Default,
      priority: 1,
    });
  }

  /// Adds a colored marker with priority.
  pub fn colored_marker_add<C>(
    &mut self, 
    coord: C, 
    symbol: &str, 
    description: &str, 
    color: DebugColor,
    priority: u32
  )
  where
    C: Into<(i32, i32)>,
  {
    let pos = coord.into();
    self.markers.insert(pos, DebugMarker {
      symbol: symbol.to_string(),
      description: description.to_string(),
      color,
      priority,
    });
  }

  /// Adds a path highlight.
  pub fn path_add<C>(&mut self, path: Vec<C>, label: &str, color: DebugColor)
  where
    C: Into<(i32, i32)>,
  {
    let coordinates = path.into_iter().map(ndarray_cg::Into::into).collect();
    self.highlights.push(DebugHighlight {
      coordinates,
      style: HighlightStyle::Outline,
      color,
      label: label.to_string(),
    });
  }

  /// Adds an area highlight.
  pub fn area_add<C>(&mut self, area: Vec<C>, label: &str, color: DebugColor, style: HighlightStyle)
  where
    C: Into<(i32, i32)>,
  {
    let coordinates = area.into_iter().map(ndarray_cg::Into::into).collect();
    self.highlights.push(DebugHighlight {
      coordinates,
      style,
      color,
      label: label.to_string(),
    });
  }

  /// Adds a text annotation.
  pub fn annotation_add<C>(&mut self, coord: C, text: &str, color: DebugColor)
  where
    C: Into<(i32, i32)>,
  {
    let position = coord.into();
    self.annotations.push(DebugAnnotation {
      position,
      text: text.to_string(),
      color,
      offset: (0, 0),
    });
  }

  /// Renders the grid as ASCII art.
  #[must_use]
  pub fn ascii_render(&self) -> String {
    let mut output = String::new();
    
    // Add header with grid information
    let _ = writeln!(output, "Debug Grid ({} x {}) - Style: {:?}",
      self.width, self.height, self.style);
    output.push_str(&"=".repeat(50));
    output.push('\n');

    match self.style {
      GridStyle::Square4 | GridStyle::Square8 => self.square_ascii_render(&mut output),
      GridStyle::Hexagonal => self.hexagonal_ascii_render(&mut output),
      GridStyle::Triangular => self.triangular_ascii_render(&mut output),
      GridStyle::Isometric => self.isometric_ascii_render(&mut output),
    }

    // Add legend
    if !self.markers.is_empty() {
      output.push('\n');
      output.push_str("Legend:\n");
      let mut markers: Vec<_> = self.markers.iter().collect();
      markers.sort_by_key(|(_, marker)| marker.priority);
      
      for ((x, y), marker) in markers {
        let _ = writeln!(output, "  {} ({}, {}) - {}",
          marker.symbol, x, y, marker.description);
      }
    }

    output
  }

  fn square_ascii_render(&self, output: &mut String) {
    // Render square grid with coordinates and markers
    for y in 0..self.height as i32 {
      // Top border
      for _x in 0..self.width as i32 {
        output.push_str("+---");
      }
      output.push_str("+\n");

      // Cell content
      for x in 0..self.width as i32 {
        output.push('|');
        
        let coord = (x, y);
        if let Some(marker) = self.markers.get(&coord) {
          let _ = write!(output, " {} ", marker.symbol);
        } else if self.is_highlighted(coord) {
          output.push_str(" # ");
        } else {
          output.push_str("   ");
        }
      }
      output.push_str("|\n");
    }

    // Bottom border
    for _ in 0..self.width {
      output.push_str("+---");
    }
    output.push_str("+\n");
  }

  fn hexagonal_ascii_render(&self, output: &mut String) {
    // Simplified hexagonal grid representation
    output.push_str("Hexagonal Grid (simplified ASCII representation):\n");
    
    for y in 0..self.height as i32 {
      // Add offset for hexagonal layout
      let offset = if y % 2 == 1 { "  " } else { "" };
      output.push_str(offset);

      for x in 0..self.width as i32 {
        let coord = (x, y);
        if let Some(marker) = self.markers.get(&coord) {
          let _ = write!(output, "/{}\\ ", marker.symbol);
        } else if self.is_highlighted(coord) {
          output.push_str("/#\\ ");
        } else {
          output.push_str("/·\\ ");
        }
      }
      output.push('\n');
    }
  }

  fn triangular_ascii_render(&self, output: &mut String) {
    output.push_str("Triangular Grid (ASCII approximation):\n");
    
    for y in 0..self.height as i32 {
      for x in 0..self.width as i32 {
        let coord = (x, y);
        if let Some(marker) = self.markers.get(&coord) {
          if (x + y) % 2 == 0 {
            let _ = write!(output, "▲{} ", marker.symbol);
          } else {
            let _ = write!(output, "▼{} ", marker.symbol);
          }
        } else if self.is_highlighted(coord) {
          output.push_str(if (x + y) % 2 == 0 { "▲# " } else { "▼# " });
        } else {
          output.push_str(if (x + y) % 2 == 0 { "▲  " } else { "▼  " });
        }
      }
      output.push('\n');
    }
  }

  fn isometric_ascii_render(&self, output: &mut String) {
    output.push_str("Isometric Grid (ASCII approximation):\n");
    
    for y in 0..self.height as i32 {
      let indent = " ".repeat((self.height as i32 - y - 1) as usize);
      output.push_str(&indent);
      
      for x in 0..self.width as i32 {
        let coord = (x, y);
        if let Some(marker) = self.markers.get(&coord) {
          let _ = write!(output, "◊{} ", marker.symbol);
        } else if self.is_highlighted(coord) {
          output.push_str("◊# ");
        } else {
          output.push_str("◊  ");
        }
      }
      output.push('\n');
    }
  }

  fn is_highlighted(&self, coord: (i32, i32)) -> bool {
    self.highlights.iter().any(|highlight| highlight.coordinates.contains(&coord))
  }

  /// Exports the grid as SVG.
  ///
  /// # Errors
  /// Returns an error when the output directory or file cannot be written.
  pub fn svg_export<P: AsRef<Path>>(&self, path: P) -> Result<(), std::io::Error> {
    if let Some(parent) = path.as_ref().parent() {
      create_dir_all(parent)?;
    }

    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);

    let cell_size = 30;
    let svg_width = self.width * cell_size + 100;
    let svg_height = self.height * cell_size + 100;

    // SVG header
    writeln!(writer, r#"<?xml version="1.0" encoding="UTF-8"?>"#)?;
    writeln!(writer, r#"<svg width="{svg_width}" height="{svg_height}" xmlns="http://www.w3.org/2000/svg">"#)?;

    // Background
    writeln!(writer, r#"<rect width="100%" height="100%" fill="white"/>"#)?;

    // Grid lines
    self.svg_grid_render(&mut writer, cell_size)?;

    // Highlights
    self.svg_highlights_render(&mut writer, cell_size)?;

    // Markers
    self.svg_markers_render(&mut writer, cell_size)?;

    // Annotations
    self.svg_annotations_render(&mut writer, cell_size)?;

    // SVG footer
    writeln!(writer, "</svg>")?;
    writer.flush()?;

    Ok(())
  }

  fn svg_grid_render(&self, writer: &mut BufWriter<File>, cell_size: usize) -> Result<(), std::io::Error> {
    let offset = 50;
    
    match self.style {
      GridStyle::Square4 | GridStyle::Square8 => {
        self.square_svg_grid_render(writer, cell_size, offset)?;
      },
      GridStyle::Hexagonal => {
        // Simplified hexagonal grid (would need proper hexagon math for production)
        for y in 0..self.height {
          for x in 0..self.width {
            let x_offset = if y % 2 == 1 { cell_size / 2 } else { 0 };
            let center_x = offset + x * cell_size + x_offset + cell_size / 2;
            let center_y = offset + y * cell_size * 3 / 4 + cell_size / 2;
            
            writeln!(writer, r#"<polygon points="{},{} {},{} {},{} {},{} {},{} {},{}" fill="none" stroke="lightgray" stroke-width="1"/>"#,
              center_x, center_y - cell_size/3,
              center_x + cell_size/3, center_y - cell_size/6,
              center_x + cell_size/3, center_y + cell_size/6,
              center_x, center_y + cell_size/3,
              center_x - cell_size/3, center_y + cell_size/6,
              center_x - cell_size/3, center_y - cell_size/6)?;
          }
        }
      },
      _ => {
        // Fix(BUG-266): this arm called `self.svg_grid_render(..)` -- itself
        // -- instead of a square-grid helper. `self.style` never changes
        // between calls, so every recursive call matched this same arm
        // again, recursing unconditionally until the stack overflowed.
        // Root cause: the comment said "Default to square grid for other
        // styles" but there was no separately-named square-grid helper to
        // delegate to, so the call was left pointing at the enclosing
        // function itself instead of the Square4/Square8 arm's logic.
        // Pitfall: a `match` arm that "falls back to another case" must call
        // a genuinely different function or change the matched value --
        // calling the same method on the same `self` from its own wildcard
        // arm is unconditional infinite recursion, not a fallback.
        self.square_svg_grid_render(writer, cell_size, offset)?;
      }
    }

    Ok(())
  }

  /// Renders axis-aligned grid lines. Shared by the square styles and used
  /// as the visual fallback for styles without dedicated SVG grid-line art.
  fn square_svg_grid_render(&self, writer: &mut BufWriter<File>, cell_size: usize, offset: usize) -> Result<(), std::io::Error> {
    // Vertical lines
    for x in 0..=self.width {
      let x_pos = offset + x * cell_size;
      writeln!(writer, r#"<line x1="{}" y1="{}" x2="{}" y2="{}" stroke="lightgray" stroke-width="1"/>"#,
        x_pos, offset, x_pos, offset + self.height * cell_size)?;
    }

    // Horizontal lines
    for y in 0..=self.height {
      let y_pos = offset + y * cell_size;
      writeln!(writer, r#"<line x1="{}" y1="{}" x2="{}" y2="{}" stroke="lightgray" stroke-width="1"/>"#,
        offset, y_pos, offset + self.width * cell_size, y_pos)?;
    }

    Ok(())
  }

  fn svg_highlights_render(&self, writer: &mut BufWriter<File>, cell_size: usize) -> Result<(), std::io::Error> {
    let offset = 50;
    
    for highlight in &self.highlights {
      let color = Self::color_to_svg(highlight.color);
      
      for &(x, y) in &highlight.coordinates {
        if x >= 0 && x < self.width as i32 && y >= 0 && y < self.height as i32 {
          let x_pos = offset + x as usize * cell_size;
          let y_pos = offset + y as usize * cell_size;
          
          match highlight.style {
            HighlightStyle::Fill => {
              writeln!(writer, r#"<rect x="{x_pos}" y="{y_pos}" width="{cell_size}" height="{cell_size}" fill="{color}" opacity="0.3"/>"#)?;
            },
            HighlightStyle::Outline => {
              writeln!(writer, r#"<rect x="{x_pos}" y="{y_pos}" width="{cell_size}" height="{cell_size}" fill="none" stroke="{color}" stroke-width="2"/>"#)?;
            },
            HighlightStyle::Dotted => {
              writeln!(writer, r#"<rect x="{x_pos}" y="{y_pos}" width="{cell_size}" height="{cell_size}" fill="none" stroke="{color}" stroke-width="2" stroke-dasharray="5,5"/>"#)?;
            },
            HighlightStyle::Animated => {
              writeln!(writer, r#"<rect x="{x_pos}" y="{y_pos}" width="{cell_size}" height="{cell_size}" fill="{color}" opacity="0.5"><animate attributeName="opacity" values="0.2;0.8;0.2" dur="2s" repeatCount="indefinite"/></rect>"#)?;
            },
          }
        }
      }
    }

    Ok(())
  }

  fn svg_markers_render(&self, writer: &mut BufWriter<File>, cell_size: usize) -> Result<(), std::io::Error> {
    let offset = 50;
    
    for (&(x, y), marker) in &self.markers {
      if x >= 0 && x < self.width as i32 && y >= 0 && y < self.height as i32 {
        let x_pos = offset + x as usize * cell_size + cell_size / 2;
        let y_pos = offset + y as usize * cell_size + cell_size / 2 + 5; // Offset for text baseline
        let color = Self::color_to_svg(marker.color);
        
        writeln!(writer, r#"<text x="{}" y="{}" text-anchor="middle" fill="{}" font-family="monospace" font-size="16" font-weight="bold">{}</text>"#,
          x_pos, y_pos, color, marker.symbol)?;
        
        // Add tooltip
        writeln!(writer, r"<title>{}</title>", marker.description)?;
      }
    }

    Ok(())
  }

  fn svg_annotations_render(&self, writer: &mut BufWriter<File>, cell_size: usize) -> Result<(), std::io::Error> {
    let offset = 50;
    
    for annotation in &self.annotations {
      let (x, y) = annotation.position;
      if x >= 0 && x < self.width as i32 && y >= 0 && y < self.height as i32 {
        let x_pos = offset + x as usize * cell_size + cell_size / 2 + annotation.offset.0 as usize;
        let y_pos = offset + y as usize * cell_size + cell_size / 4 + annotation.offset.1 as usize;
        let color = Self::color_to_svg(annotation.color);
        
        writeln!(writer, r#"<text x="{}" y="{}" fill="{}" font-family="sans-serif" font-size="12">{}</text>"#,
          x_pos, y_pos, color, annotation.text)?;
      }
    }

    Ok(())
  }

  fn color_to_svg(color: DebugColor) -> &'static str {
    match color {
      DebugColor::Default => "black",
      DebugColor::Red => "red",
      DebugColor::Green => "green",
      DebugColor::Blue => "blue",
      DebugColor::Yellow => "orange",
      DebugColor::Purple => "purple",
      DebugColor::Orange => "darkorange",
      DebugColor::Gray => "gray",
    }
  }

  /// Clears all debug information.
  pub fn clear(&mut self) {
    self.markers.clear();
    self.highlights.clear();
    self.annotations.clear();
  }
}

impl Default for GridRenderer {
  fn default() -> Self {
    Self::new()
  }
}

/// Pathfinding debug visualizer.
pub struct PathfindingDebugger {
  grid_renderer: GridRenderer,
  path_costs: HashMap<(i32, i32), u32>,
  visited_nodes: Vec<(i32, i32)>,
  open_nodes: Vec<(i32, i32)>,
  obstacles: Vec<(i32, i32)>,
}

impl PathfindingDebugger {
  /// Creates a new pathfinding debugger.
  #[must_use]
  pub fn new(width: usize, height: usize) -> Self {
    Self {
      grid_renderer: GridRenderer::new().with_size(width, height),
      path_costs: HashMap::new(),
      visited_nodes: Vec::new(),
      open_nodes: Vec::new(),
      obstacles: Vec::new(),
    }
  }

  /// Adds an obstacle at the specified coordinate.
  pub fn obstacle_add<C>(&mut self, coord: C)
  where
    C: Into<(i32, i32)>,
  {
    let pos = coord.into();
    self.obstacles.push(pos);
    self.grid_renderer.colored_marker_add(pos, "X", "Obstacle", DebugColor::Red, 10);
  }

  /// Sets the start position.
  pub fn start_set<C>(&mut self, coord: C)
  where
    C: Into<(i32, i32)>,
  {
    let pos = coord.into();
    self.grid_renderer.colored_marker_add(pos, "S", "Start", DebugColor::Green, 20);
  }

  /// Sets the goal position.
  pub fn goal_set<C>(&mut self, coord: C)
  where
    C: Into<(i32, i32)>,
  {
    let pos = coord.into();
    self.grid_renderer.colored_marker_add(pos, "G", "Goal", DebugColor::Blue, 20);
  }

  /// Adds a path to visualize.
  pub fn path_add<C>(&mut self, path: Vec<C>, label: &str)
  where
    C: Into<(i32, i32)>,
  {
    let path_coords: Vec<(i32, i32)> = path.into_iter().map(ndarray_cg::Into::into).collect();
    
    // Add path markers
    for (i, &coord) in path_coords.iter().enumerate() {
      if i > 0 && i < path_coords.len() - 1 {
        self.grid_renderer.colored_marker_add(coord, "·", "Path point", DebugColor::Yellow, 5);
      }
    }

    self.grid_renderer.path_add(path_coords, label, DebugColor::Yellow);
  }

  /// Adds visited nodes from pathfinding algorithm.
  pub fn visited_nodes_add<C>(&mut self, nodes: Vec<C>)
  where
    C: Into<(i32, i32)>,
  {
    self.visited_nodes = nodes.into_iter().map(ndarray_cg::Into::into).collect();
    self.grid_renderer.area_add(self.visited_nodes.clone(), "Visited", DebugColor::Gray, HighlightStyle::Fill);
  }

  /// Adds open nodes from pathfinding algorithm.
  pub fn open_nodes_add<C>(&mut self, nodes: Vec<C>)
  where
    C: Into<(i32, i32)>,
  {
    self.open_nodes = nodes.into_iter().map(ndarray_cg::Into::into).collect();
    self.grid_renderer.area_add(self.open_nodes.clone(), "Open", DebugColor::Orange, HighlightStyle::Dotted);
  }

  /// Sets cost information for pathfinding visualization.
  pub fn costs_set(&mut self, costs: HashMap<(i32, i32), u32>) {
    self.path_costs = costs;
    
    // Add cost annotations
    for (&coord, &cost) in &self.path_costs {
      if cost > 1 && !self.obstacles.contains(&coord) {
        self.grid_renderer.annotation_add(coord, &cost.to_string(), DebugColor::Purple);
      }
    }
  }

  /// Renders the pathfinding debug view as ASCII.
  #[must_use]
  pub fn ascii_render(&self) -> String {
    self.grid_renderer.ascii_render()
  }

  /// Exports the pathfinding debug view as SVG.
  ///
  /// # Errors
  /// Returns an error when the underlying grid export fails.
  pub fn svg_export<P: AsRef<Path>>(&self, path: P) -> Result<(), std::io::Error> {
    self.grid_renderer.svg_export(path)
  }
}

/// ECS component inspector for debugging entity data.
pub struct ECSInspector {
  entity_data: HashMap<u32, EntityDebugInfo>,
  component_counts: HashMap<String, usize>,
  system_timings: HashMap<String, Duration>,
}

/// Debug information for a single entity.
#[derive(Debug, Clone)]
pub struct EntityDebugInfo {
  /// Entity ID
  pub id: u32,
  /// Component type names
  pub components: Vec<String>,
  /// Current position (if applicable)
  pub position: Option<(i32, i32)>,
  /// Custom debug data
  pub data: HashMap<String, String>,
}

impl ECSInspector {
  /// Creates a new ECS inspector.
  #[must_use]
  pub fn new() -> Self {
    Self {
      entity_data: HashMap::new(),
      component_counts: HashMap::new(),
      system_timings: HashMap::new(),
    }
  }

  /// Records entity information.
  pub fn entity_record(&mut self, entity: EntityDebugInfo) {
    // BUG-347 task/bug/347_ecs_inspector_entity_record_inflates_component_counts.md
    // -- re-recording an entity inflated component_counts; fix below.
    // Fix(BUG-347): decrement the previous entry's component counts (if
    // entity.id was already recorded) before applying the new entity's
    // counts, so re-recording the same entity_id does not permanently
    // inflate component_counts.
    // Root cause: every call incremented component_counts for the new
    // entity's components, then unconditionally overwrote entity_data via
    // HashMap::insert -- the prior call's contribution to component_counts
    // was never removed, and no entity_remove/unrecord method existed to
    // correct it either.
    // Pitfall: a counter incremented on every call of a "record" method that
    // can be called more than once for the same identity needs a matching
    // decrement for whatever it is replacing -- otherwise re-recording
    // silently inflates the counter forever, with no panic to surface it.
    if let Some(previous) = self.entity_data.get(&entity.id) {
      for component in &previous.components {
        if let Some(count) = self.component_counts.get_mut(component) {
          *count = count.saturating_sub(1);
          if *count == 0 {
            self.component_counts.remove(component);
          }
        }
      }
    }

    for component in &entity.components {
      *self.component_counts.entry(component.clone()).or_insert(0) += 1;
    }
    self.entity_data.insert(entity.id, entity);
  }

  /// Records system execution time.
  pub fn system_timing_record(&mut self, system_name: String, duration: Duration) {
    self.system_timings.insert(system_name, duration);
  }

  /// Gets the number of entities currently tracked.
  #[must_use]
  pub fn entity_count(&self) -> usize {
    self.entity_data.len()
  }

  /// Gets entity information by ID.
  #[must_use]
  pub fn entity_get(&self, id: u32) -> Option<&EntityDebugInfo> {
    self.entity_data.get(&id)
  }

  /// Gets all entity IDs.
  #[must_use]
  pub fn entity_ids(&self) -> Vec<u32> {
    self.entity_data.keys().copied().collect()
  }

  /// Generates a debug report.
  #[must_use]
  pub fn report_generate(&self) -> String {
    let mut report = String::new();
    
    report.push_str("ECS Inspector Report\n");
    report.push_str("===================\n\n");

    // Entity summary
    let _ = writeln!(report, "Total Entities: {}", self.entity_data.len());
    
    // Component statistics
    report.push_str("\nComponent Statistics:\n");
    let mut components: Vec<_> = self.component_counts.iter().collect();
    components.sort_by_key(|(_, count)| *count);
    for (component, count) in components.iter().rev() {
      let _ = writeln!(report, "  {component}: {count} entities");
    }

    // System timings
    if !self.system_timings.is_empty() {
      report.push_str("\nSystem Performance:\n");
      let mut timings: Vec<_> = self.system_timings.iter().collect();
      timings.sort_by_key(|(_, duration)| *duration);
      for (system, duration) in timings.iter().rev() {
        let _ = writeln!(report, "  {}: {:.2}ms", system, duration.as_secs_f64() * 1000.0);
      }
    }

    // Detailed entity information
    report.push_str("\nDetailed Entity Information:\n");
    let mut entities: Vec<_> = self.entity_data.values().collect();
    entities.sort_by_key(|e| e.id);
    
    for entity in entities.iter().take(10) { // Limit to first 10 for readability
      let _ = writeln!(report, "\nEntity {}:", entity.id);
      let _ = writeln!(report, "  Components: {}", entity.components.join(", "));
      if let Some(pos) = entity.position {
        let _ = writeln!(report, "  Position: ({}, {})", pos.0, pos.1);
      }
      for (key, value) in &entity.data {
        let _ = writeln!(report, "  {key}: {value}");
      }
    }

    if self.entity_data.len() > 10 {
      let _ = writeln!(report, "\n... and {} more entities", self.entity_data.len() - 10);
    }

    report
  }

  /// Exports entity data as JSON.
  ///
  /// Fix(BUG-478): every string value (component names, system-timing
  /// names, entity data keys/values) is now escaped via
  /// `utils::json_string_escape`, and the output now includes each entity's
  /// own record (id, components, position, custom data), aligning its scope
  /// with `report_generate`.
  /// Root cause: the original implementation built JSON via bare
  /// `format!("\"{name}\": {count}")`-style interpolation with no escaping
  /// at all (a component name or entity data value containing `"` or `\`
  /// produced invalid JSON), and never iterated `self.entity_data` -- only
  /// the two aggregate maps, omitting the per-entity detail
  /// `report_generate` already includes.
  /// Pitfall: an available `serde_json` dependency is not automatically
  /// usable from every module that could benefit from it -- `serde_json` is
  /// only a crate dependency behind this crate's `serialization` feature,
  /// while `debug` is gated only by `enabled`; wiring `serde_json` in here
  /// would make `debug` implicitly require `serialization` (or force
  /// `serde_json` to become a non-optional dependency of the base `enabled`
  /// feature). Neither felt like the natural fix for this pass, so this
  /// stays a hand-rolled -- but now correctly escaped and scope-aligned --
  /// writer; revisit if this crate's dependency structure changes.
  #[must_use]
  pub fn json_export(&self) -> String {
    let mut json = String::from("{\n");
    let _ = writeln!(json, "  \"total_entities\": {},", self.entity_data.len());

    json.push_str("  \"component_counts\": {\n");
    let mut components: Vec<_> = self.component_counts.iter().collect();
    components.sort_by_key(|(name, _)| (*name).clone());
    let component_entries: Vec<String> = components.iter()
      .map(|(name, count)| format!("    {}: {count}", utils::json_string_escape(name)))
      .collect();
    json.push_str(&component_entries.join(",\n"));
    json.push_str("\n  },\n");

    json.push_str("  \"system_timings\": {\n");
    let mut timings: Vec<_> = self.system_timings.iter().collect();
    timings.sort_by_key(|(name, _)| (*name).clone());
    let timing_entries: Vec<String> = timings.iter()
      .map(|(name, duration)| format!("    {}: {:.2}", utils::json_string_escape(name), duration.as_secs_f64() * 1000.0))
      .collect();
    json.push_str(&timing_entries.join(",\n"));
    json.push_str("\n  },\n");

    json.push_str("  \"entities\": [\n");
    let mut entities: Vec<_> = self.entity_data.values().collect();
    entities.sort_by_key(|e| e.id);
    let entity_entries: Vec<String> = entities.iter()
      .map(|entity| {
        let components = entity.components.iter()
          .map(|c| utils::json_string_escape(c))
          .collect::<Vec<_>>()
          .join(", ");
        let position = match entity.position {
          Some((x, y)) => format!("{{ \"x\": {x}, \"y\": {y} }}"),
          None => "null".to_string(),
        };
        let mut data_entries: Vec<_> = entity.data.iter().collect();
        data_entries.sort_by_key(|(key, _)| (*key).clone());
        let data = data_entries.iter()
          .map(|(key, value)| format!("{}: {}", utils::json_string_escape(key), utils::json_string_escape(value)))
          .collect::<Vec<_>>()
          .join(", ");
        format!(
          "    {{ \"id\": {}, \"components\": [{components}], \"position\": {position}, \"data\": {{{data}}} }}",
          entity.id,
        )
      })
      .collect();
    json.push_str(&entity_entries.join(",\n"));
    json.push_str("\n  ]\n");

    json.push('}');
    json
  }
}

impl Default for ECSInspector {
  fn default() -> Self {
    Self::new()
  }
}

/// Performance profiler for tracking frame times and bottlenecks.
pub struct PerformanceProfiler {
  frame_times: VecDeque<Duration>,
  system_times: HashMap<String, VecDeque<Duration>>,
  memory_samples: VecDeque<MemorySample>,
  start_time: Instant,
  frame_count: u64,
}

/// Memory usage sample.
#[derive(Debug, Clone, Copy)]
pub struct MemorySample {
  /// Timestamp of the sample
  pub timestamp: Duration,
  /// Estimated memory usage in bytes
  pub memory_usage: u64,
  /// Number of active entities
  pub entity_count: u32,
}

impl PerformanceProfiler {
  /// Creates a new performance profiler.
  #[must_use]
  pub fn new() -> Self {
    Self {
      frame_times: VecDeque::with_capacity(1000),
      system_times: HashMap::new(),
      memory_samples: VecDeque::with_capacity(1000),
      start_time: Instant::now(),
      frame_count: 0,
    }
  }

  /// Records a frame time.
  pub fn frame_time_record(&mut self, duration: Duration) {
    self.frame_times.push_back(duration);
    if self.frame_times.len() > 1000 {
      self.frame_times.pop_front();
    }
    self.frame_count += 1;
  }

  /// Records system execution time.
  pub fn system_time_record(&mut self, system_name: String, duration: Duration) {
    let times = self.system_times.entry(system_name).or_insert_with(|| VecDeque::with_capacity(100));
    times.push_back(duration);
    if times.len() > 100 {
      times.pop_front();
    }
  }

  /// Records memory usage sample.
  pub fn memory_sample_record(&mut self, memory_usage: u64, entity_count: u32) {
    let sample = MemorySample {
      timestamp: self.start_time.elapsed(),
      memory_usage,
      entity_count,
    };
    self.memory_samples.push_back(sample);
    if self.memory_samples.len() > 1000 {
      self.memory_samples.pop_front();
    }
  }

  /// Gets current performance statistics.
  #[must_use]
  pub fn stats_get(&self) -> PerformanceStats {
    let avg_frame_time = if self.frame_times.is_empty() {
      Duration::ZERO
    } else {
      self.frame_times.iter().sum::<Duration>() / self.frame_times.len() as u32
    };

    let min_frame_time = self.frame_times.iter().min().copied().unwrap_or(Duration::ZERO);
    let max_frame_time = self.frame_times.iter().max().copied().unwrap_or(Duration::ZERO);

    let fps = if avg_frame_time.as_secs_f64() > 0.0 {
      1.0 / avg_frame_time.as_secs_f64()
    } else {
      0.0
    };

    let current_memory = self.memory_samples.back().map_or(0, |s| s.memory_usage);
    let current_entities = self.memory_samples.back().map_or(0, |s| s.entity_count);

    PerformanceStats {
      avg_frame_time,
      min_frame_time,
      max_frame_time,
      fps,
      frame_count: self.frame_count,
      current_memory,
      current_entities,
      uptime: self.start_time.elapsed(),
    }
  }

  /// Generates a performance report.
  #[must_use]
  pub fn report_generate(&self) -> String {
    let stats = self.stats_get();
    let mut report = String::new();

    report.push_str("Performance Profile Report\n");
    report.push_str("=========================\n\n");

    let _ = writeln!(report, "Uptime: {:.1}s", stats.uptime.as_secs_f64());
    let _ = writeln!(report, "Frame Count: {}", stats.frame_count);
    let _ = writeln!(report, "Average FPS: {:.1}", stats.fps);
    let _ = writeln!(report, "Frame Time: {:.2}ms (avg), {:.2}ms (min), {:.2}ms (max)",
      stats.avg_frame_time.as_secs_f64() * 1000.0,
      stats.min_frame_time.as_secs_f64() * 1000.0,
      stats.max_frame_time.as_secs_f64() * 1000.0);
    
    let _ = writeln!(report, "Memory Usage: {} KB", stats.current_memory / 1024);
    let _ = writeln!(report, "Active Entities: {}", stats.current_entities);

    if !self.system_times.is_empty() {
      report.push_str("\nSystem Performance:\n");
      for (system, times) in &self.system_times {
        if !times.is_empty() {
          let avg = times.iter().sum::<Duration>() / times.len() as u32;
          let max = times.iter().max().copied().unwrap_or(Duration::ZERO);
          let _ = writeln!(report, "  {}: {:.2}ms avg, {:.2}ms max",
            system, 
            avg.as_secs_f64() * 1000.0,
            max.as_secs_f64() * 1000.0);
        }
      }
    }

    report
  }

  /// Exports performance data as CSV for analysis.
  ///
  /// # Errors
  /// Returns an error when the CSV file cannot be created or written.
  pub fn csv_export<P: AsRef<Path>>(&self, path: P) -> Result<(), std::io::Error> {
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);

    // Header
    writeln!(writer, "timestamp_ms,frame_time_ms,memory_kb,entity_count")?;

    // Data
    //
    // Fix(BUG-481): `memory_samples` is populated independently of, and not
    // necessarily in lockstep with, `frame_times` -- indexing the shorter
    // deque past its own length used to silently default to a zero-valued
    // `MemorySample`, which is indistinguishable in the output from a
    // genuine "0 bytes, 0 entities" sample. A row with no corresponding
    // memory sample now leaves those two CSV fields blank instead.
    // Root cause: `.get(i).copied().unwrap_or(MemorySample { .. 0 .. })`
    // treated "no sample recorded at this index" the same as "a sample of
    // zero was recorded", collapsing two different facts into one value.
    // Pitfall: zipping two independently-populated collections by index and
    // defaulting a missing entry to a real, in-range value (here, zero) is
    // never actually "safe" -- zero is frequently also a legitimate
    // observed value, so the default is silently indistinguishable from
    // real data. Default to `Option`/blank instead, never a same-typed
    // sentinel value.
    for (i, frame_time) in self.frame_times.iter().enumerate() {
      let timestamp_ms = i as f64 * 16.67; // Approximate 60 FPS timing
      let frame_time_ms = frame_time.as_secs_f64() * 1000.0;

      match self.memory_samples.get(i) {
        Some(memory_sample) => writeln!(writer, "{:.2},{:.2},{},{}",
          timestamp_ms,
          frame_time_ms,
          memory_sample.memory_usage / 1024,
          memory_sample.entity_count)?,
        None => writeln!(writer, "{timestamp_ms:.2},{frame_time_ms:.2},,")?,
      }
    }

    writer.flush()?;
    Ok(())
  }
}

/// Performance statistics snapshot.
#[derive(Debug, Clone)]
pub struct PerformanceStats {
  /// Average frame time
  pub avg_frame_time: Duration,
  /// Minimum frame time recorded
  pub min_frame_time: Duration,
  /// Maximum frame time recorded
  pub max_frame_time: Duration,
  /// Current frames per second
  pub fps: f64,
  /// Total frame count
  pub frame_count: u64,
  /// Current memory usage in bytes
  pub current_memory: u64,
  /// Current number of entities
  pub current_entities: u32,
  /// Total uptime
  pub uptime: Duration,
}

impl Default for PerformanceProfiler {
  fn default() -> Self {
    Self::new()
  }
}

/// Coordinate conversion trait for debug rendering.
pub trait IntoDebugCoord {
  /// Converts the coordinate to a debug-friendly (i32, i32) tuple.
  fn into_debug_coord(self) -> (i32, i32);
}

// Implement for common coordinate types
impl IntoDebugCoord for (i32, i32) {
  fn into_debug_coord(self) -> (i32, i32) {
    self
  }
}

impl IntoDebugCoord for (f32, f32) {
  fn into_debug_coord(self) -> (i32, i32) {
    (self.0 as i32, self.1 as i32)
  }
}

impl IntoDebugCoord for (usize, usize) {
  fn into_debug_coord(self) -> (i32, i32) {
    (self.0 as i32, self.1 as i32)
  }
}

/// Utility functions for debugging.
pub mod utils {
  use super::Duration;
  use std::fmt::Write as _;

  /// Creates a simple ASCII art representation of a 2D boolean array.
  #[must_use]
  pub fn bool_grid_render(grid: &[Vec<bool>], true_char: char, false_char: char) -> String {
    let mut output = String::new();
    for row in grid {
      for &cell in row {
        output.push(if cell { true_char } else { false_char });
        output.push(' ');
      }
      output.push('\n');
    }
    output
  }

  /// Formats a duration for human-readable display.
  #[must_use]
  pub fn duration_format(duration: Duration) -> String {
    let micros = duration.as_micros();
    if micros < 1000 {
      format!("{micros}μs")
    } else if micros < 1_000_000 {
      format!("{:.1}ms", duration.as_secs_f64() * 1000.0)
    } else {
      format!("{:.2}s", duration.as_secs_f64())
    }
  }

  /// Escapes a string for embedding in a JSON string literal, per the JSON
  /// spec's required escapes (quote, backslash, and the C0 control
  /// characters). Returns the value already wrapped in quotes.
  #[must_use]
  pub fn json_string_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
      match c {
        '"' => out.push_str("\\\""),
        '\\' => out.push_str("\\\\"),
        '\n' => out.push_str("\\n"),
        '\r' => out.push_str("\\r"),
        '\t' => out.push_str("\\t"),
        c if (c as u32) < 0x20 => { let _ = write!(out, "\\u{:04x}", c as u32); },
        c => out.push(c),
      }
    }
    out.push('"');
    out
  }

  /// Formats memory usage for human-readable display.
  #[must_use]
  pub fn memory_format(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
      format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
      format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
      format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
      format!("{bytes} B")
    }
  }
}

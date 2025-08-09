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
//! renderer.add_marker(Coordinate::<Axial, Pointy>::new(2, 3), "S", "Start position");
//! renderer.add_marker(Coordinate::<Axial, Pointy>::new(7, 5), "G", "Goal position");
//!
//! // Render as ASCII art
//! println!("{}", renderer.render_ascii());
//!
//! // Export as SVG
//! renderer.export_svg("debug_grid.svg").expect("Failed to export SVG");
//! ```

use std::collections::{HashMap, VecDeque};
use std::time::{Instant, Duration};
use std::fs::{File, create_dir_all};
use std::io::{Write, BufWriter};
use std::path::Path;
use crate::coordinates::{Distance, Neighbors};

/// Visual debugging renderer for coordinate grids.
pub struct GridRenderer {
  width: usize,
  height: usize,
  style: GridStyle,
  markers: HashMap<(i32, i32), DebugMarker>,
  highlights: Vec<DebugHighlight>,
  annotations: Vec<DebugAnnotation>,
}

/// Style options for grid rendering.
#[derive(Debug, Clone, Copy)]
pub enum GridStyle {
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
#[derive(Debug, Clone)]
pub struct DebugMarker {
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

impl GridRenderer {
  /// Creates a new grid renderer.
  pub fn new() -> Self {
    Self {
      width: 20,
      height: 15,
      style: GridStyle::Square4,
      markers: HashMap::new(),
      highlights: Vec::new(),
      annotations: Vec::new(),
    }
  }

  /// Sets the grid size.
  pub fn with_size(mut self, width: usize, height: usize) -> Self {
    self.width = width;
    self.height = height;
    self
  }

  /// Sets the grid style.
  pub fn with_style(mut self, style: GridStyle) -> Self {
    self.style = style;
    self
  }

  /// Adds a debug marker at the specified coordinate.
  pub fn add_marker<C>(&mut self, coord: C, symbol: &str, description: &str)
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
  pub fn add_colored_marker<C>(
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
  pub fn add_path<C>(&mut self, path: Vec<C>, label: &str, color: DebugColor)
  where
    C: Into<(i32, i32)>,
  {
    let coordinates = path.into_iter().map(|c| c.into()).collect();
    self.highlights.push(DebugHighlight {
      coordinates,
      style: HighlightStyle::Outline,
      color,
      label: label.to_string(),
    });
  }

  /// Adds an area highlight.
  pub fn add_area<C>(&mut self, area: Vec<C>, label: &str, color: DebugColor, style: HighlightStyle)
  where
    C: Into<(i32, i32)>,
  {
    let coordinates = area.into_iter().map(|c| c.into()).collect();
    self.highlights.push(DebugHighlight {
      coordinates,
      style,
      color,
      label: label.to_string(),
    });
  }

  /// Adds a text annotation.
  pub fn add_annotation<C>(&mut self, coord: C, text: &str, color: DebugColor)
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
  pub fn render_ascii(&self) -> String {
    let mut output = String::new();
    
    // Add header with grid information
    output.push_str(&format!("Debug Grid ({} x {}) - Style: {:?}\n", 
      self.width, self.height, self.style));
    output.push_str(&"=".repeat(50));
    output.push('\n');

    match self.style {
      GridStyle::Square4 | GridStyle::Square8 => self.render_square_ascii(&mut output),
      GridStyle::Hexagonal => self.render_hexagonal_ascii(&mut output),
      GridStyle::Triangular => self.render_triangular_ascii(&mut output),
      GridStyle::Isometric => self.render_isometric_ascii(&mut output),
    }

    // Add legend
    if !self.markers.is_empty() {
      output.push('\n');
      output.push_str("Legend:\n");
      let mut markers: Vec<_> = self.markers.iter().collect();
      markers.sort_by_key(|(_, marker)| marker.priority);
      
      for ((x, y), marker) in markers {
        output.push_str(&format!("  {} ({}, {}) - {}\n", 
          marker.symbol, x, y, marker.description));
      }
    }

    output
  }

  fn render_square_ascii(&self, output: &mut String) {
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
          output.push_str(&format!(" {} ", marker.symbol));
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

  fn render_hexagonal_ascii(&self, output: &mut String) {
    // Simplified hexagonal grid representation
    output.push_str("Hexagonal Grid (simplified ASCII representation):\n");
    
    for y in 0..self.height as i32 {
      // Add offset for hexagonal layout
      let offset = if y % 2 == 1 { "  " } else { "" };
      output.push_str(offset);

      for x in 0..self.width as i32 {
        let coord = (x, y);
        if let Some(marker) = self.markers.get(&coord) {
          output.push_str(&format!("/{}\\ ", marker.symbol));
        } else if self.is_highlighted(coord) {
          output.push_str("/#\\ ");
        } else {
          output.push_str("/·\\ ");
        }
      }
      output.push('\n');
    }
  }

  fn render_triangular_ascii(&self, output: &mut String) {
    output.push_str("Triangular Grid (ASCII approximation):\n");
    
    for y in 0..self.height as i32 {
      for x in 0..self.width as i32 {
        let coord = (x, y);
        if let Some(marker) = self.markers.get(&coord) {
          if (x + y) % 2 == 0 {
            output.push_str(&format!("▲{} ", marker.symbol));
          } else {
            output.push_str(&format!("▼{} ", marker.symbol));
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

  fn render_isometric_ascii(&self, output: &mut String) {
    output.push_str("Isometric Grid (ASCII approximation):\n");
    
    for y in 0..self.height as i32 {
      let indent = " ".repeat((self.height as i32 - y - 1) as usize);
      output.push_str(&indent);
      
      for x in 0..self.width as i32 {
        let coord = (x, y);
        if let Some(marker) = self.markers.get(&coord) {
          output.push_str(&format!("◊{} ", marker.symbol));
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
  pub fn export_svg<P: AsRef<Path>>(&self, path: P) -> Result<(), std::io::Error> {
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
    writeln!(writer, r#"<svg width="{}" height="{}" xmlns="http://www.w3.org/2000/svg">"#, 
      svg_width, svg_height)?;

    // Background
    writeln!(writer, r#"<rect width="100%" height="100%" fill="white"/>"#)?;

    // Grid lines
    self.render_svg_grid(&mut writer, cell_size)?;

    // Highlights
    self.render_svg_highlights(&mut writer, cell_size)?;

    // Markers
    self.render_svg_markers(&mut writer, cell_size)?;

    // Annotations
    self.render_svg_annotations(&mut writer, cell_size)?;

    // SVG footer
    writeln!(writer, "</svg>")?;
    writer.flush()?;

    Ok(())
  }

  fn render_svg_grid(&self, writer: &mut BufWriter<File>, cell_size: usize) -> Result<(), std::io::Error> {
    let offset = 50;
    
    match self.style {
      GridStyle::Square4 | GridStyle::Square8 => {
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
        // Default to square grid for other styles
        self.render_svg_grid(writer, cell_size)?;
      }
    }

    Ok(())
  }

  fn render_svg_highlights(&self, writer: &mut BufWriter<File>, cell_size: usize) -> Result<(), std::io::Error> {
    let offset = 50;
    
    for highlight in &self.highlights {
      let color = self.color_to_svg(highlight.color);
      
      for &(x, y) in &highlight.coordinates {
        if x >= 0 && x < self.width as i32 && y >= 0 && y < self.height as i32 {
          let x_pos = offset + x as usize * cell_size;
          let y_pos = offset + y as usize * cell_size;
          
          match highlight.style {
            HighlightStyle::Fill => {
              writeln!(writer, r#"<rect x="{}" y="{}" width="{}" height="{}" fill="{}" opacity="0.3"/>"#,
                x_pos, y_pos, cell_size, cell_size, color)?;
            },
            HighlightStyle::Outline => {
              writeln!(writer, r#"<rect x="{}" y="{}" width="{}" height="{}" fill="none" stroke="{}" stroke-width="2"/>"#,
                x_pos, y_pos, cell_size, cell_size, color)?;
            },
            HighlightStyle::Dotted => {
              writeln!(writer, r#"<rect x="{}" y="{}" width="{}" height="{}" fill="none" stroke="{}" stroke-width="2" stroke-dasharray="5,5"/>"#,
                x_pos, y_pos, cell_size, cell_size, color)?;
            },
            HighlightStyle::Animated => {
              writeln!(writer, r#"<rect x="{}" y="{}" width="{}" height="{}" fill="{}" opacity="0.5"><animate attributeName="opacity" values="0.2;0.8;0.2" dur="2s" repeatCount="indefinite"/></rect>"#,
                x_pos, y_pos, cell_size, cell_size, color)?;
            },
          }
        }
      }
    }

    Ok(())
  }

  fn render_svg_markers(&self, writer: &mut BufWriter<File>, cell_size: usize) -> Result<(), std::io::Error> {
    let offset = 50;
    
    for (&(x, y), marker) in &self.markers {
      if x >= 0 && x < self.width as i32 && y >= 0 && y < self.height as i32 {
        let x_pos = offset + x as usize * cell_size + cell_size / 2;
        let y_pos = offset + y as usize * cell_size + cell_size / 2 + 5; // Offset for text baseline
        let color = self.color_to_svg(marker.color);
        
        writeln!(writer, r#"<text x="{}" y="{}" text-anchor="middle" fill="{}" font-family="monospace" font-size="16" font-weight="bold">{}</text>"#,
          x_pos, y_pos, color, marker.symbol)?;
        
        // Add tooltip
        writeln!(writer, r#"<title>{}</title>"#, marker.description)?;
      }
    }

    Ok(())
  }

  fn render_svg_annotations(&self, writer: &mut BufWriter<File>, cell_size: usize) -> Result<(), std::io::Error> {
    let offset = 50;
    
    for annotation in &self.annotations {
      let (x, y) = annotation.position;
      if x >= 0 && x < self.width as i32 && y >= 0 && y < self.height as i32 {
        let x_pos = offset + x as usize * cell_size + cell_size / 2 + annotation.offset.0 as usize;
        let y_pos = offset + y as usize * cell_size + cell_size / 4 + annotation.offset.1 as usize;
        let color = self.color_to_svg(annotation.color);
        
        writeln!(writer, r#"<text x="{}" y="{}" fill="{}" font-family="sans-serif" font-size="12">{}</text>"#,
          x_pos, y_pos, color, annotation.text)?;
      }
    }

    Ok(())
  }

  fn color_to_svg(&self, color: DebugColor) -> &'static str {
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
  pub fn add_obstacle<C>(&mut self, coord: C)
  where
    C: Into<(i32, i32)>,
  {
    let pos = coord.into();
    self.obstacles.push(pos);
    self.grid_renderer.add_colored_marker(pos, "X", "Obstacle", DebugColor::Red, 10);
  }

  /// Sets the start position.
  pub fn set_start<C>(&mut self, coord: C)
  where
    C: Into<(i32, i32)>,
  {
    let pos = coord.into();
    self.grid_renderer.add_colored_marker(pos, "S", "Start", DebugColor::Green, 20);
  }

  /// Sets the goal position.
  pub fn set_goal<C>(&mut self, coord: C)
  where
    C: Into<(i32, i32)>,
  {
    let pos = coord.into();
    self.grid_renderer.add_colored_marker(pos, "G", "Goal", DebugColor::Blue, 20);
  }

  /// Adds a path to visualize.
  pub fn add_path<C>(&mut self, path: Vec<C>, label: &str)
  where
    C: Into<(i32, i32)>,
  {
    let path_coords: Vec<(i32, i32)> = path.into_iter().map(|c| c.into()).collect();
    
    // Add path markers
    for (i, &coord) in path_coords.iter().enumerate() {
      if i > 0 && i < path_coords.len() - 1 {
        self.grid_renderer.add_colored_marker(coord, "·", "Path point", DebugColor::Yellow, 5);
      }
    }

    self.grid_renderer.add_path(path_coords, label, DebugColor::Yellow);
  }

  /// Adds visited nodes from pathfinding algorithm.
  pub fn add_visited_nodes<C>(&mut self, nodes: Vec<C>)
  where
    C: Into<(i32, i32)>,
  {
    self.visited_nodes = nodes.into_iter().map(|c| c.into()).collect();
    self.grid_renderer.add_area(self.visited_nodes.clone(), "Visited", DebugColor::Gray, HighlightStyle::Fill);
  }

  /// Adds open nodes from pathfinding algorithm.
  pub fn add_open_nodes<C>(&mut self, nodes: Vec<C>)
  where
    C: Into<(i32, i32)>,
  {
    self.open_nodes = nodes.into_iter().map(|c| c.into()).collect();
    self.grid_renderer.add_area(self.open_nodes.clone(), "Open", DebugColor::Orange, HighlightStyle::Dotted);
  }

  /// Sets cost information for pathfinding visualization.
  pub fn set_costs(&mut self, costs: HashMap<(i32, i32), u32>) {
    self.path_costs = costs;
    
    // Add cost annotations
    for (&coord, &cost) in &self.path_costs {
      if cost > 1 && !self.obstacles.contains(&coord) {
        self.grid_renderer.add_annotation(coord, &cost.to_string(), DebugColor::Purple);
      }
    }
  }

  /// Renders the pathfinding debug view as ASCII.
  pub fn render_ascii(&self) -> String {
    self.grid_renderer.render_ascii()
  }

  /// Exports the pathfinding debug view as SVG.
  pub fn export_svg<P: AsRef<Path>>(&self, path: P) -> Result<(), std::io::Error> {
    self.grid_renderer.export_svg(path)
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
  pub fn new() -> Self {
    Self {
      entity_data: HashMap::new(),
      component_counts: HashMap::new(),
      system_timings: HashMap::new(),
    }
  }

  /// Records entity information.
  pub fn record_entity(&mut self, entity: EntityDebugInfo) {
    for component in &entity.components {
      *self.component_counts.entry(component.clone()).or_insert(0) += 1;
    }
    self.entity_data.insert(entity.id, entity);
  }

  /// Records system execution time.
  pub fn record_system_timing(&mut self, system_name: String, duration: Duration) {
    self.system_timings.insert(system_name, duration);
  }

  /// Gets the number of entities currently tracked.
  pub fn entity_count(&self) -> usize {
    self.entity_data.len()
  }

  /// Gets entity information by ID.
  pub fn get_entity(&self, id: u32) -> Option<&EntityDebugInfo> {
    self.entity_data.get(&id)
  }

  /// Gets all entity IDs.
  pub fn entity_ids(&self) -> Vec<u32> {
    self.entity_data.keys().copied().collect()
  }

  /// Generates a debug report.
  pub fn generate_report(&self) -> String {
    let mut report = String::new();
    
    report.push_str("ECS Inspector Report\n");
    report.push_str("===================\n\n");

    // Entity summary
    report.push_str(&format!("Total Entities: {}\n", self.entity_data.len()));
    
    // Component statistics
    report.push_str("\nComponent Statistics:\n");
    let mut components: Vec<_> = self.component_counts.iter().collect();
    components.sort_by_key(|(_, count)| *count);
    for (component, count) in components.iter().rev() {
      report.push_str(&format!("  {}: {} entities\n", component, count));
    }

    // System timings
    if !self.system_timings.is_empty() {
      report.push_str("\nSystem Performance:\n");
      let mut timings: Vec<_> = self.system_timings.iter().collect();
      timings.sort_by_key(|(_, duration)| *duration);
      for (system, duration) in timings.iter().rev() {
        report.push_str(&format!("  {}: {:.2}ms\n", system, duration.as_secs_f64() * 1000.0));
      }
    }

    // Detailed entity information
    report.push_str("\nDetailed Entity Information:\n");
    let mut entities: Vec<_> = self.entity_data.values().collect();
    entities.sort_by_key(|e| e.id);
    
    for entity in entities.iter().take(10) { // Limit to first 10 for readability
      report.push_str(&format!("\nEntity {}:\n", entity.id));
      report.push_str(&format!("  Components: {}\n", entity.components.join(", ")));
      if let Some(pos) = entity.position {
        report.push_str(&format!("  Position: ({}, {})\n", pos.0, pos.1));
      }
      for (key, value) in &entity.data {
        report.push_str(&format!("  {}: {}\n", key, value));
      }
    }

    if self.entity_data.len() > 10 {
      report.push_str(&format!("\n... and {} more entities\n", self.entity_data.len() - 10));
    }

    report
  }

  /// Exports entity data as JSON.
  pub fn export_json(&self) -> String {
    // Simplified JSON export (in real implementation would use serde_json)
    let mut json = String::from("{\n");
    json.push_str(&format!("  \"total_entities\": {},\n", self.entity_data.len()));
    
    json.push_str("  \"component_counts\": {\n");
    let component_entries: Vec<String> = self.component_counts.iter()
      .map(|(name, count)| format!("    \"{}\": {}", name, count))
      .collect();
    json.push_str(&component_entries.join(",\n"));
    json.push_str("\n  },\n");
    
    json.push_str("  \"system_timings\": {\n");
    let timing_entries: Vec<String> = self.system_timings.iter()
      .map(|(name, duration)| format!("    \"{}\": {:.2}", name, duration.as_secs_f64() * 1000.0))
      .collect();
    json.push_str(&timing_entries.join(",\n"));
    json.push_str("\n  }\n");
    
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
  pub fn record_frame_time(&mut self, duration: Duration) {
    self.frame_times.push_back(duration);
    if self.frame_times.len() > 1000 {
      self.frame_times.pop_front();
    }
    self.frame_count += 1;
  }

  /// Records system execution time.
  pub fn record_system_time(&mut self, system_name: String, duration: Duration) {
    let times = self.system_times.entry(system_name).or_insert_with(|| VecDeque::with_capacity(100));
    times.push_back(duration);
    if times.len() > 100 {
      times.pop_front();
    }
  }

  /// Records memory usage sample.
  pub fn record_memory_sample(&mut self, memory_usage: u64, entity_count: u32) {
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
  pub fn get_stats(&self) -> PerformanceStats {
    let avg_frame_time = if !self.frame_times.is_empty() {
      self.frame_times.iter().sum::<Duration>() / self.frame_times.len() as u32
    } else {
      Duration::ZERO
    };

    let min_frame_time = self.frame_times.iter().min().copied().unwrap_or(Duration::ZERO);
    let max_frame_time = self.frame_times.iter().max().copied().unwrap_or(Duration::ZERO);

    let fps = if avg_frame_time.as_secs_f64() > 0.0 {
      1.0 / avg_frame_time.as_secs_f64()
    } else {
      0.0
    };

    let current_memory = self.memory_samples.back().map(|s| s.memory_usage).unwrap_or(0);
    let current_entities = self.memory_samples.back().map(|s| s.entity_count).unwrap_or(0);

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
  pub fn generate_report(&self) -> String {
    let stats = self.get_stats();
    let mut report = String::new();

    report.push_str("Performance Profile Report\n");
    report.push_str("=========================\n\n");

    report.push_str(&format!("Uptime: {:.1}s\n", stats.uptime.as_secs_f64()));
    report.push_str(&format!("Frame Count: {}\n", stats.frame_count));
    report.push_str(&format!("Average FPS: {:.1}\n", stats.fps));
    report.push_str(&format!("Frame Time: {:.2}ms (avg), {:.2}ms (min), {:.2}ms (max)\n",
      stats.avg_frame_time.as_secs_f64() * 1000.0,
      stats.min_frame_time.as_secs_f64() * 1000.0,
      stats.max_frame_time.as_secs_f64() * 1000.0));
    
    report.push_str(&format!("Memory Usage: {} KB\n", stats.current_memory / 1024));
    report.push_str(&format!("Active Entities: {}\n", stats.current_entities));

    if !self.system_times.is_empty() {
      report.push_str("\nSystem Performance:\n");
      for (system, times) in &self.system_times {
        if !times.is_empty() {
          let avg = times.iter().sum::<Duration>() / times.len() as u32;
          let max = times.iter().max().copied().unwrap_or(Duration::ZERO);
          report.push_str(&format!("  {}: {:.2}ms avg, {:.2}ms max\n",
            system, 
            avg.as_secs_f64() * 1000.0,
            max.as_secs_f64() * 1000.0));
        }
      }
    }

    report
  }

  /// Exports performance data as CSV for analysis.
  pub fn export_csv<P: AsRef<Path>>(&self, path: P) -> Result<(), std::io::Error> {
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);

    // Header
    writeln!(writer, "timestamp_ms,frame_time_ms,memory_kb,entity_count")?;

    // Data
    for (i, frame_time) in self.frame_times.iter().enumerate() {
      let timestamp_ms = i as f64 * 16.67; // Approximate 60 FPS timing
      let memory_sample = self.memory_samples.get(i).copied().unwrap_or(MemorySample {
        timestamp: Duration::from_millis(timestamp_ms as u64),
        memory_usage: 0,
        entity_count: 0,
      });
      
      writeln!(writer, "{:.2},{:.2},{},{}", 
        timestamp_ms,
        frame_time.as_secs_f64() * 1000.0,
        memory_sample.memory_usage / 1024,
        memory_sample.entity_count)?;
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
  use super::*;

  /// Creates a simple ASCII art representation of a 2D boolean array.
  pub fn render_bool_grid(grid: &[Vec<bool>], true_char: char, false_char: char) -> String {
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
  pub fn format_duration(duration: Duration) -> String {
    let micros = duration.as_micros();
    if micros < 1000 {
      format!("{}μs", micros)
    } else if micros < 1_000_000 {
      format!("{:.1}ms", duration.as_secs_f64() * 1000.0)
    } else {
      format!("{:.2}s", duration.as_secs_f64())
    }
  }

  /// Formats memory usage for human-readable display.
  pub fn format_memory(bytes: u64) -> String {
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
      format!("{} B", bytes)
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::time::Duration;

  #[test]
  fn test_grid_renderer_creation() {
    let renderer = GridRenderer::new()
      .with_size(10, 8)
      .with_style(GridStyle::Hexagonal);
    
    assert_eq!(renderer.width, 10);
    assert_eq!(renderer.height, 8);
    assert!(matches!(renderer.style, GridStyle::Hexagonal));
  }

  #[test]
  fn test_grid_renderer_markers() {
    let mut renderer = GridRenderer::new();
    renderer.add_marker((5, 3), "S", "Start position");
    renderer.add_colored_marker((8, 6), "G", "Goal", DebugColor::Blue, 10);
    
    assert_eq!(renderer.markers.len(), 2);
    assert!(renderer.markers.contains_key(&(5, 3)));
    assert!(renderer.markers.contains_key(&(8, 6)));
  }

  #[test]
  fn test_pathfinding_debugger() {
    let mut debugger = PathfindingDebugger::new(10, 10);
    
    debugger.set_start((0, 0));
    debugger.set_goal((9, 9));
    debugger.add_obstacle((5, 5));
    debugger.add_path(vec![(0, 0), (1, 1), (2, 2), (3, 3)], "Test Path");
    
    let output = debugger.render_ascii();
    assert!(output.contains("Start"));
    assert!(output.contains("Goal"));
    assert!(output.contains("Obstacle"));
  }

  #[test]
  fn test_ecs_inspector() {
    let mut inspector = ECSInspector::new();
    
    let entity = EntityDebugInfo {
      id: 42,
      components: vec!["Position".to_string(), "Health".to_string()],
      position: Some((10, 20)),
      data: vec![("level".to_string(), "5".to_string())].into_iter().collect(),
    };
    
    inspector.record_entity(entity);
    inspector.record_system_timing("MovementSystem".to_string(), Duration::from_millis(5));
    
    let report = inspector.generate_report();
    assert!(report.contains("Entity 42"));
    assert!(report.contains("Position"));
    assert!(report.contains("MovementSystem"));
  }

  #[test]
  fn test_performance_profiler() {
    let mut profiler = PerformanceProfiler::new();
    
    profiler.record_frame_time(Duration::from_millis(16));
    profiler.record_frame_time(Duration::from_millis(18));
    profiler.record_system_time("RenderSystem".to_string(), Duration::from_millis(8));
    profiler.record_memory_sample(1024 * 1024, 100); // 1MB, 100 entities
    
    let stats = profiler.get_stats();
    assert_eq!(stats.frame_count, 2);
    assert!(stats.fps > 0.0);
    assert_eq!(stats.current_memory, 1024 * 1024);
    assert_eq!(stats.current_entities, 100);
  }

  #[test]
  fn test_debug_utilities() {
    let grid = vec![
      vec![true, false, true],
      vec![false, true, false],
      vec![true, true, false],
    ];
    
    let output = utils::render_bool_grid(&grid, '#', '.');
    assert!(output.contains('#'));
    assert!(output.contains('.'));
    
    let duration = Duration::from_micros(1500);
    let formatted = utils::format_duration(duration);
    assert!(formatted.contains("1.5ms"));
    
    let memory = utils::format_memory(1536 * 1024); // 1.5 MB
    assert!(memory.contains("1.5") && memory.contains("MB"));
  }

  #[test]
  fn test_coordinate_conversion() {
    let int_coord = (5, 10);
    let float_coord = (5.7, 10.3);
    let usize_coord = (5usize, 10usize);
    
    assert_eq!(int_coord.into_debug_coord(), (5, 10));
    assert_eq!(float_coord.into_debug_coord(), (5, 10));
    assert_eq!(usize_coord.into_debug_coord(), (5, 10));
  }
}
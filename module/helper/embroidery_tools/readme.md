# 🧵 embroidery_tools

> **Comprehensive embroidery file format support for reading and writing stitch patterns**

A robust library for handling embroidery files in various formats. Supports reading, writing, and manipulating stitch patterns for embroidery machines and design software. Built with precision and reliability for professional embroidery workflows.

## ✨ Features

### 📁 **File Format Support**
- **PEC Format** - Brother embroidery machine format (read/write)
- **PES Format** - Brother/Babylock embroidery format (v1 & v6)
- **Cross-Platform** - Works across different embroidery machine brands
- **Version Detection** - Automatic format version identification

### 🔧 **Core Capabilities**
- **Pattern Reading** - Extract stitch data, colors, and metadata
- **Pattern Writing** - Generate machine-compatible embroidery files
- **Stitch Analysis** - Examine pattern properties and statistics
- **Color Management** - Handle thread colors and palettes

### 📊 **Data Processing**
- **Stitch Instructions** - Jump, move, color change, and trim commands
- **Coordinate Systems** - Accurate positioning and scaling
- **Thread Colors** - RGB and palette-based color management
- **Pattern Metadata** - Design information and machine settings

## 📦 Installation

Add to your `Cargo.toml`:
```toml
embroidery_tools = { workspace = true }
```

## 🚀 Quick Start

### Reading Embroidery Files

```rust
use embroidery_tools::*;

fn read_pattern() -> Result<(), Box<dyn std::error::Error>> {
  // Read PES file
  let pattern = pes::read_file("design.pes")?;
  
  println!("Pattern info:");
  println!("  Stitches: {}", pattern.stitch_count());
  println!("  Colors: {}", pattern.color_count());
  println!("  Size: {}x{} mm", pattern.width(), pattern.height());
  
  // Access stitch data
  for stitch in pattern.stitches() {
    match stitch.command {
      StitchCommand::Normal => {
        println!("Stitch at ({}, {})", stitch.x, stitch.y);
      },
      StitchCommand::Jump => {
        println!("Jump to ({}, {})", stitch.x, stitch.y);
      },
      StitchCommand::ColorChange => {
        println!("Color change at ({}, {})", stitch.x, stitch.y);
      },
    }
  }
  
  Ok(())
}
```

### Writing Embroidery Files

```rust
use embroidery_tools::*;

fn create_pattern() -> Result<(), Box<dyn std::error::Error>> {
  // Create new pattern
  let mut pattern = EmbroideryPattern::new();
  
  // Add color palette
  pattern.add_color(Color::rgb(255, 0, 0));   // Red
  pattern.add_color(Color::rgb(0, 255, 0));   // Green  
  pattern.add_color(Color::rgb(0, 0, 255));   // Blue
  
  // Add stitches
  pattern.add_stitch(Stitch::normal(0, 0));
  pattern.add_stitch(Stitch::normal(100, 0));
  pattern.add_stitch(Stitch::normal(100, 100));
  pattern.add_stitch(Stitch::color_change(100, 100));
  pattern.add_stitch(Stitch::normal(0, 100));
  pattern.add_stitch(Stitch::normal(0, 0));
  
  // Write to PES format
  pes::write_file(&pattern, "output.pes", PesVersion::V6)?;
  
  // Write to PEC format
  pec::write_file(&pattern, "output.pec")?;
  
  Ok(())
}
```

## 📖 API Reference

### Supported Formats

| Format | Read | Write | Versions | Description |
|--------|------|-------|----------|-------------|
| **PES** | ✅ | ✅ | v1, v6 | Brother/Babylock embroidery format |
| **PEC** | ✅ | ✅ | - | Brother embroidery machine format |

### Core Types

| Type | Description | Use Case |
|------|-------------|----------|
| `EmbroideryPattern` | Complete pattern data | Pattern manipulation and storage |
| `Stitch` | Individual stitch point | Building stitch sequences |
| `Color` | Thread color information | Color palette management |
| `StitchCommand` | Stitch type/instruction | Machine command interpretation |

### Pattern Operations

```rust
// Pattern analysis
let bounds = pattern.bounds();
let stitch_count = pattern.stitch_count();
let color_count = pattern.color_count();

// Pattern modification
pattern.scale(2.0);                    // Scale by factor
pattern.translate(50, 25);             // Move pattern
pattern.rotate(std::f32::consts::PI);  // Rotate pattern
pattern.optimize();                    // Remove redundant stitches
```

## 🎯 Use Cases

### Professional Embroidery
- **Design Software Integration** - Import/export for embroidery design tools
- **Production Workflow** - Convert between different machine formats
- **Quality Control** - Analyze and validate embroidery patterns
- **Archive Management** - Organize and catalog design collections

### Industrial Applications
- **Automated Production** - Generate patterns from CAD/vector data
- **Format Conversion** - Bridge different embroidery machine systems
- **Pattern Analysis** - Calculate thread usage and production time
- **Custom Tooling** - Build specialized embroidery workflows

### Educational & Research
- **Pattern Study** - Analyze traditional and modern embroidery techniques
- **Algorithm Development** - Research optimal stitch path generation
- **Format Documentation** - Understand embroidery file structures
- **Tool Development** - Create new embroidery software solutions

## 🔧 Current Status & Roadmap

### ✅ Implemented Features
- **PEC Format** - Full read/write support
- **PES Format** - Versions 1 and 6 support
- **Basic Pattern Operations** - Create, read, modify patterns
- **Color Management** - Handle thread colors and palettes

### 🚧 Planned Features
- **Pattern Normalization** - Automatic format compatibility fixes
- **Additional Formats** - DST, JEF, EXP, and other formats
- **Advanced Editing** - Cut, copy, paste, merge operations
- **Optimization Algorithms** - Minimize jumps and thread changes
- **Preview Generation** - Render patterns for display

### ⚠️ Current Limitations
- Pattern editing capabilities are basic
- Some stitch instructions may need normalization before writing
- Limited to PES and PEC formats currently
- No built-in pattern optimization algorithms

## 📊 Technical Details

### File Format Specifications
The library handles the binary formats according to official specifications:
- **PES v1**: Original Brother format with basic stitch data
- **PES v6**: Extended format with additional metadata and features  
- **PEC**: Compressed Brother format optimized for machine storage

### Coordinate Systems
- Internal coordinates use standard Cartesian system (mm)
- Automatic conversion to/from machine-specific coordinate systems
- Proper handling of origin points and scaling factors

### Thread Color Handling
```rust
// RGB color specification
let red = Color::rgb(255, 0, 0);

// Palette-based colors
let thread = Color::palette_index(5);

// Named thread colors (if supported by format)
let rayon = Color::thread("Madeira Rayon 1147");
```

## 🛠️ Integration Examples

### With Image Processing
```rust
// Convert vector graphics to embroidery
use embroidery_tools::*;

fn vectorize_to_embroidery(svg_path: &str) -> Result<EmbroideryPattern, Box<dyn std::error::Error>> {
  // Parse SVG and convert to stitch pattern
  let mut pattern = EmbroideryPattern::new();
  
  // Add stitches following vector paths
  // (Implementation would depend on vector processing library)
  
  Ok(pattern)
}
```

### Batch Processing
```rust
// Convert multiple files
use embroidery_tools::*;
use std::fs;

fn convert_directory(input_dir: &str, output_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
  for entry in fs::read_dir(input_dir)? {
    let path = entry?.path();
    if path.extension() == Some("pes".as_ref()) {
      let pattern = pes::read_file(&path)?;
      let output_path = format!("{}/{}.pec", output_dir, path.file_stem().unwrap().to_str().unwrap());
      pec::write_file(&pattern, &output_path)?;
    }
  }
  Ok(())
}
```

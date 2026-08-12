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
use embroidery_tools::format::pes;
use embroidery_tools::stitch_instruction::Instruction;

fn read_pattern() -> Result<(), Box<dyn std::error::Error>> {
  // Read a PES file
  let emb = pes::file_read("design.pes")?;

  let (min_x, min_y, max_x, max_y) = emb.bounds();
  println!("Pattern info:");
  println!("  Stitches: {}", emb.stitches().len());
  println!("  Threads: {}", emb.threads().len());
  println!("  Bounds: ({min_x}, {min_y}) to ({max_x}, {max_y})");

  // Access stitch data
  for stitch in emb.stitches() {
    match stitch.instruction {
      Instruction::Stitch => println!("Stitch at ({}, {})", stitch.x, stitch.y),
      Instruction::Jump => println!("Jump to ({}, {})", stitch.x, stitch.y),
      Instruction::ColorChange => println!("Color change at ({}, {})", stitch.x, stitch.y),
      _ => {}
    }
  }

  Ok(())
}
```

`embroidery_tools` uses the `mod_interface` layering convention: types are exported from their
own submodule (`embroidery_file::EmbroideryFile`, `thread::{Color, Thread}`,
`stitch_instruction::{Instruction, Stitch}`, `format::{pec, pes}`), never re-exported at the crate
root — `use embroidery_tools::*;` resolves nothing usable. Import from the specific submodule.

### Writing Embroidery Files

```rust
use std::fs::File;
use std::io::BufWriter;
use embroidery_tools::embroidery_file::EmbroideryFile;
use embroidery_tools::thread::{Color, Thread};
use embroidery_tools::format::{pec, pes};
use embroidery_tools::format::pes::PESVersion;

fn create_pattern() -> Result<(), Box<dyn std::error::Error>> {
  // Build a new pattern
  let mut emb = EmbroideryFile::new();

  // Register the thread palette
  emb.thread_add(Thread { color: Color { r: 255, g: 0, b: 0 }, ..Default::default() }); // Red
  emb.thread_add(Thread { color: Color { r: 0, g: 255, b: 0 }, ..Default::default() }); // Green

  // Add stitches — coordinates passed to these helpers are relative to the previous point
  emb.stitch(0, 0);
  emb.stitch(100, 0);
  emb.stitch(0, 100);
  emb.color_change(0, 0);
  emb.stitch(-100, 0);
  emb.trim();
  emb.end();

  // There is no path-based `write_file()` convenience — writers take any `Write + Seek`,
  // so open the destination yourself.
  let mut pes_out = BufWriter::new(File::create("output.pes")?);
  pes::write(&mut emb, &mut pes_out, PESVersion::V6)?;

  let mut pec_out = BufWriter::new(File::create("output.pec")?);
  pec::write(&mut emb, &mut pec_out)?;

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

| Type | Module | Description | Use Case |
|------|--------|-------------|----------|
| `EmbroideryFile` | `embroidery_file` | Complete pattern: stitches, threads, metadata | Building and inspecting patterns |
| `Stitch` | `stitch_instruction` | One instruction plus its `(x, y)` coordinates | Building stitch sequences |
| `Instruction` | `stitch_instruction` | Stitch/Jump/Trim/ColorChange/etc. kind (`#[non_exhaustive]`) | Interpreting or emitting machine commands |
| `Thread` | `thread` | One palette entry — color plus catalog metadata | Building the thread palette |
| `Color` | `thread` | Plain `{ r, g, b }` triple | Thread color values |

### Pattern Operations

```rust
// Pattern inspection
let (min_x, min_y, max_x, max_y) = emb.bounds();
let stitch_count = emb.stitches().len();
let thread_count = emb.threads().len();

// Pattern normalization — fixes instruction encoding before writing or after reading
emb.color_count_fix();                      // ensure enough threads for every color change
emb.stop_interpolate_as_duplicate_color();  // encode Stop as a duplicated color change
emb.duplicate_color_interpolate_as_stop();  // decode a duplicated color change back to Stop
```

There is currently no geometric transform API (scale/translate/rotate) — see Current Limitations
below.

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
- **Stitch Encoding Normalization** - `color_count_fix()`, `stop_interpolate_as_duplicate_color()`,
  `duplicate_color_interpolate_as_stop()` (called explicitly, not automatic)

### 🚧 Planned Features
- **Geometric Transforms** - Scale, translate, rotate a pattern
- **Additional Formats** - DST, JEF, EXP, and other formats
- **Advanced Editing** - Cut, copy, paste, merge operations
- **Optimization Algorithms** - Minimize jumps and thread changes
- **Preview Generation** - Render patterns for display

### ⚠️ Current Limitations
- No geometric transform API (scale/translate/rotate) yet
- Stitch-encoding normalization must be called explicitly, not applied automatically
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
use embroidery_tools::thread::{Color, Thread};

// Colors are plain RGB triples — no constructor methods
let red = Color { r: 255, g: 0, b: 0 };

// Threads pair a color with catalog metadata
let rayon = Thread {
  color: red,
  description: "Madeira Rayon 1147".into(),
  catalog_number: "1147".into(),
  ..Default::default()
};
```

## 🛠️ Integration Examples

### With Image Processing
```rust
// Convert vector graphics to embroidery
use embroidery_tools::embroidery_file::EmbroideryFile;

fn vectorize_to_embroidery(_svg_path: &str) -> Result<EmbroideryFile, Box<dyn std::error::Error>> {
  // Parse SVG and convert to stitch pattern
  let mut emb = EmbroideryFile::new();

  // Add stitches following vector paths
  // (Implementation would depend on vector processing library)

  Ok(emb)
}
```

### Batch Processing
```rust
// Convert multiple files
use embroidery_tools::format::{pec, pes};
use std::fs::{self, File};
use std::io::BufWriter;

fn convert_directory(input_dir: &str, output_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
  for entry in fs::read_dir(input_dir)? {
    let path = entry?.path();
    if path.extension() == Some("pes".as_ref()) {
      let mut emb = pes::file_read(&path)?;
      let stem = path.file_stem().unwrap().to_str().unwrap();
      let mut out = BufWriter::new(File::create(format!("{output_dir}/{stem}.pec"))?);
      pec::write(&mut emb, &mut out)?;
    }
  }
  Ok(())
}
```

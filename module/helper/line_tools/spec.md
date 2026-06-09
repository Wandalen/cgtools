# Line Tools Engine Specification

**Name:** Line Tools  
**Version:** 1.2 (Rulebook Compliant)  
**Date:** 2026-06-09

## Table of Contents

1. [Project Goal](#1-project-goal)
2. [Problem Solved](#2-problem-solved)
3. [Ubiquitous Language](#3-ubiquitous-language)
4. [Use Cases & Applications](#4-use-cases--applications)
5. [Deliverables](#5-deliverables)
6. [Functional Requirements](#6-functional-requirements)
7. [Non-Functional Requirements](#7-non-functional-requirements)
8. [Dependency Architecture](#8-dependency-architecture)
9. [Module Structure](#9-module-structure)  
10. [Conformance Checklist](#10-conformance-checklist)  
11. [Corner cases](#11-corner-cases)  

---

## 1. Project Goal

Create a high-performance Rust crate (`line_tools`) that provides comprehensive tools for rendering lines.

## 2. Problem Solved
`line_tools` provides a single, robust library for rendering lines built on proven architectural patterns from the wTools ecosystem.

## 3. Ubiquitous Language

| Term | Definition |
|------|------------|
| **Line** | A list of points that make up a line |
| **Join** | The geometry type to join to straight line segments together |
| **Cap** | The geometry type to cover the end points of the line |
| **Mesh** | The collection of WebGL buffers and shader programs used to render a line |
| **Program** | A compiled and linked vertex/fragment shader pair with its VAO and uniforms |
| **Distance** | The cumulative arc length from the line's start to a given point; enabled by the `distance` feature |
| **Vertex Color** | An optional per-point color attribute used to shade the line |
| **World Units** | Line-width mode where width is measured in world space (shrinks with distance) instead of screen space |
| **Alpha to Coverage** | An anti-aliasing technique using MSAA coverage instead of alpha testing for line edges |
| **Dash Pattern** | A repeating on/off segment pattern (`DashPattern::V1`–`V4`) applied along a 3D line's length |
| **Dash Offset** | A scalar shift applied to the start of the dash pattern |


## 4. Use Cases & Applications

### 4.1 Primary Use Cases

| Use Case | Complexity |
|----------|------------|
| **Data Visualization** | Customizable line thickness and style for representing data values, clean rendering for charts and graphs.|
| **Game Development**| Drawing UI elements (health bars, maps), creating visual effects (lasers, trails), debugging tools. |
| **Art and Illustration** | Smooth, customizable strokes with varying thickness, creative control over line ends and corners for artistic expression. |


### 4.2 Performance Characteristics

| Operation | O |
|-----------|-----------|
| **Add a point** | O(1) |
| **Remove a point** | O(n) | 
| **Create a mesh** | O(n) | 

## 5. Deliverables

1. **Published Rust Crate**: Compliant with workspace dependency management
2. **Source Code**: Following strict codestyle and design rulebooks  
3. **API Documentation**: Generated via `cargo doc` with complete coverage
4. **Usage Tutorials**: Step-by-step guides for common use cases

## 6. Functional Requirements

- **FR-1**: Different implementations of lines should be feature seperated
- **FR-2**: 3D line should decrease in size with distance from the camera
- **FR-3**: Line should support editing of any points from the list
- **FR-4**: 3D line should support dashed rendering with selectable `DashPattern` variants (`V1`–`V4`) and an adjustable `dash_offset`, available under the `distance` feature
- **FR-5**: Line should support optional per-vertex colors
- **FR-6**: 3D line width should be expressible in either screen-space or world units
- **FR-7**: Line edges should be anti-aliasable via either alpha testing or alpha-to-coverage
- **FR-8**: 2D line should support configurable cap (Butt, Round, Square) and join (Miter, Round, Bevel) styles
- **FR-9**: Line styling types (caps, joins) should be serializable via the `serialization` feature

## 7. Non-Functional Requirements
- **NFR-1**: 100% documentation coverage via `cargo doc`
- **NFR-2**: All functions use noun-verb naming order
- **NFR-3**: 100% adherence to Codestyle Rulebook formatting

## 8. Dependency Architecture

### 8.1 Required wTools Dependencies
```toml
[dependencies]
# Core wTools ecosystem
minwebgl = { workspace = true, optional = true }
ndarray_cg = { workspace = true, optional = true }
mod_interface = { workspace = true, optional = true }
serde = { workspace = true, features = [ "derive" ], optional = true }
rustc-hash = { workspace = true }

web-sys = { workspace = true, optional = true, features = [
  'WebGlActiveInfo',
]}

# Development and testing
test_tools = { workspace = true, optional = true }
```

### 8.2 Current Architecture (Implemented)
```
src/
├── lib.rs  
├── d2.rs
├── d3.rs
├── caps.rs
├── joins.rs
├── helpers.rs
├── mesh.rs
├── program.rs
├── uniform.rs
├── d2/
│   ├── line.rs
│   └── shaders/    # GLSL body, terminal, join and cap shaders
└── d3/
    ├── line.rs
    └── shaders/    # GLSL main vertex/fragment shaders

```

## 9. Module Structure

Each module follows the strict mod_interface pattern:

```rust
mod private
{

}

::mod_interface::mod_interface!
{
  own use ::mod_interface::mod_interface;

  /// A layer for 2D graphics-related functionalities.
  layer d2;
  /// A layer for 3D graphics-related functionalities.
  layer d3;

  /// A layer dedicated to line join styles (e.g., miter, bevel, round).
  layer joins;
  /// A layer dedicated to line cap styles (e.g., butt, round, square).
  layer caps;

  /// A layer for mesh generation and manipulation.
  layer mesh;
  /// A layer for shader programs and related functionality.
  layer program;

  /// A layer for helper functions and utilities used by other modules.
  layer helpers;

  /// Module for handling uniform operations
  layer uniform;
}
```

## 10. Conformance Checklist

- ✅ **FR-1**: Different implementations of lines should be feature seperated
- ✅ **FR-2**: 3D line should decrease in size with distance from the camera
- ✅ **FR-3**: Line should support editing of any points from the list
- ✅ **FR-4**: 3D line should support dashed rendering with selectable `DashPattern` variants and an adjustable `dash_offset`
- ✅ **FR-5**: Line should support optional per-vertex colors
- ✅ **FR-6**: 3D line width should be expressible in either screen-space or world units
- ✅ **FR-7**: Line edges should be anti-aliasable via either alpha testing or alpha-to-coverage
- ✅ **FR-8**: 2D line should support configurable cap and join styles
- ✅ **FR-9**: Line styling types (caps, joins) should be serializable via the `serialization` feature

- ✅ **NFR-1**: 100% documentation coverage via `cargo doc`
- ⏳ **NFR-2**: All functions use noun-verb naming order ( 3D line follows it; 2D line still uses verb-noun, e.g. `create_mesh`, `add_point` )
- ✅ **NFR-3**: 100% adherence to Codestyle Rulebook formatting

## 11. Corner cases
- ✅ **1**: Overlapping geometry when using blending - joins, caps, segment body are draw as seperate geometry, causing a visible overlap when using blending
- ✅ **2**: With a small angle and big enough width, two neighbouring segments begin to overlap
- ❌ **3**: When points are very close to eachother and line width is much bigger than the distance between the points - segments begin to overlap a lot
- ✅ **4**: When neighbouring points are placed at the same position - the line brakes due to zero vector length
- ❌ **5**: Side effect of the solution for the second corner case - unusual ovelapping between non neighbouring segments
- ✅ **6**: When neighbouring segments are parallel to each other, the division by zero happens causing the line to break
- ✅ **7**: As line gets wider, the UV coordinates shrink and the flips the sign
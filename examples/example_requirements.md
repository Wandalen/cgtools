# CGTools Example Requirements

This document defines the requirements and standards for creating new examples in the CGTools project. Following these guidelines ensures consistency, quality, and maintainability across all examples.

## Table of Contents

- [Example Structure](#example-structure)
- [File Requirements](#file-requirements)
- [Readme Template Guide](#readme-template-guide)

---

## Example Structure

Each example must follow this standardized directory structure:

```
example_name/
  src/
    main.rs            # Main application code (required)
  shaders/             # Shader files (if applicable)
    shader.vert        # Vertex shader
    shader.frag        # Fragment shader
  Cargo.toml           # Project dependencies (required)
  index.html           # HTML entry point (required for WebGL/WebGPU)
  readme.md            # Example documentation (required)
  showcase.png/jpg     # Preview image (required)
```

### Directory Categories

Examples are organized into the following categories:

- **minwebgl/** - WebGL 2.0 examples (browser-based)
- **minwebgpu/** - WebGPU examples (browser-based)
- **minwgpu/** - wgpu examples (native rendering)
- **math/** - Mathematical computation examples

---

## File Requirements

### 1. Cargo.toml

Every example must include a `Cargo.toml` file with:

```toml
[package]
name = "example_name"
version = "0.1.0"
edition = "2021" # or "2024" for newer examples

[lints]
workspace = true  # Use workspace-level lints

[dependencies]
# Use workspace dependencies
minwebgl = { workspace = true }
# or minwebgpu = { workspace = true }
# or minwgpu = { workspace = true }

# Additional dependencies as needed
```

**Key Points:**
- Use workspace dependencies (e.g., `{ workspace = true }`)
- Include `[lints]` section to inherit workspace linting rules
- Use edition 2021 or 2024
- Keep dependencies minimal and focused

### 2. index.html (WebGL/WebGPU only)

For browser-based examples, include a minimal `index.html`:

```html
<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>Example Name</title>
    <style>
        body { margin: 0; overflow: hidden; }
        canvas { width: 100%; height: 100%; display: block; }
    </style>
</head>
<body>
    <canvas id="canvas"></canvas>
</body>
</html>
```

### 3. src/main.rs

The main application file must:
- Include clear module documentation
- Follow workspace linting standards
- Be well-commented for educational purposes
- Include error handling
- Be production-ready code quality

Example structure:

```rust
//! Brief description of the example
//!
//! Detailed explanation of the technique demonstrated

use minwebgl::*;

fn main()
{
    // Clear, well-documented code
}
```

### 4. showcase.png or showcase.jpg

Every example **must** include a showcase image:

- **Format:** PNG, JPEG, GIF
- **Recommended size:** 1920x1080 or 1280x720
- **Aspect ratio:** 16:9 preferred
- **Quality:** High quality, representative of the example's output
- **Filename:** `showcase.png`, `showcase.jpg`, `showcase.gif` (lowercase)

The showcase image should:
- Clearly demonstrate the example's visual output
- Be captured in release mode for best quality
- Show the most interesting/representative state
- Be properly lit and composed

### 5. readme.md

**Required.** Every example must include comprehensive documentation. See [README Template Guide](#readme-template-guide) below for detailed instructions.

---

## README Template

All example readme.md files **must** follow the standardized template format defined in [demo_readme_example.md](./demo_readme_example.md).

### Step-by-Step Guide

#### 1. Title

```markdown
# Example Title
```

**Requirements:**
- Use title case (capitalize major words)
- Be descriptive and clear
- Match the example's purpose
- Convert folder name to human-readable format

**Examples:**
- `trivial` � `# Trivial WebGL Example`
- `hexagonal_grid` � `# Hexagonal Grid`
- `deferred_shading` � `# Deferred Shading`
- `hello_triangle` � `# Hello Triangle`

#### 2. Keywords

```markdown
**Keywords:** keyword1, keyword2, keyword3, keyword4
```

**Requirements:**
- Include 3-6 relevant keywords
- Separate with commas
- Use title case or proper nouns
- Include the rendering API (WebGL2, WebGPU, wgpu)
- Include the main technique/concept

**Keyword Categories:**
- **API/Technology:** WebGL2, WebGPU, wgpu, Rust, WASM
- **Technique:** PBR, Deferred Shading, Instancing, Transparency
- **Domain:** Rendering, Animation, Procedural, Pathfinding
- **Level:** Tutorial, Advanced, Optimization

**Examples:**
```markdown
**Keywords:** WebGL2, Hexagons, Grids, Pathfinding
**Keywords:** WebGPU, Deferred Rendering, Lighting, PBR
**Keywords:** Transparency, WebGL2, OIT, Blending
**Keywords:** wgpu, Rust, Tutorial, Getting Started
```

#### 3. Description

Write **2-3 paragraphs** that explain:

**Paragraph 1:** What and Why
- What the example demonstrates
- The main technique or concept
- Why this technique is important/useful

**Paragraph 2:** How and Technical Details
- How the technique works
- Key technical implementation details
- Any special algorithms or approaches

**Paragraph 3 (optional):** Applications and Use Cases
- Real-world applications
- When to use this technique
- Performance characteristics or limitations

**Example Structure:**

```markdown
This demo demonstrates **order-independent transparency** using
the **weighted blended OIT** technique in WebGL2. Traditional
transparency rendering requires sorting all transparent objects
back-to-front, which becomes costly or incorrect when geometry
is interpenetrating. Weighted blended OIT provides an approximate
but visually convincing solution without sorting.

It accumulates color and transparency in separate buffers using
floating-point blending, then combines them in a final compositing
pass. The technique uses multiple render targets to store weighted
color and alpha values.

This approach works efficiently on modern GPUs and is suitable for
real-time scenes such as particle systems, glass objects, and
volumetric effects.
```

**Writing Guidelines:**
- Use technical language but remain accessible
- Bold key concepts on first mention
- Be specific about techniques and algorithms
- Explain the "why" not just the "what"
- Keep paragraphs focused (3-5 sentences each)

#### 4. Showcase Image

```markdown
![image](./showcase.png)
```

**Requirements:**
- Use relative path to showcase image
- Use `./showcase.png`, `./showcase.jpg`, `./showcase.gif`
- Include the `./` prefix for clarity
- Use lowercase filename

#### 5. How to Run Link

```markdown
**[How to run](relative_path_to_how_to_run.md)**
```

#### 6. References

```markdown
**References:**

* [Reference Name 1]
* [Reference Name 2]
* [Reference Name 3]

[Reference Name 1]: https://example.com/page1
[Reference Name 2]: https://example.com/page2
[Reference Name 3]: https://example.com/page3
```

**Requirements:**
- Include at least 2 references
- Use descriptive reference names
- Use markdown reference-style links
- Provide high-quality, relevant resources

# 🎨 vectorizer

> **Advanced raster-to-vector image conversion with intelligent layer separation**

A powerful vectorization tool that converts bitmap images into scalable vector graphics (SVG). Built on top of VTracer with enhanced algorithms for better quality output and customizable processing modes. Perfect for creating clean vector graphics from photographs, artwork, and logos.

## ✨ Features

### 🔄 **Dual Processing Modes**
- **Clusters Mode** - Original VTracer clustering with enhancements
- **Layers Mode** - Advanced layer-based separation for superior quality
- **Customizable Pipeline** - Fine-tune parameters for your specific use case

### 🎯 **Intelligent Processing**
- **Color Precision Control** - Adjustable bit depth for color comparison
- **Speckle Filtering** - Remove noise and small artifacts
- **Background Removal** - Automatic or manual background elimination
- **Gradient Handling** - Smooth color transitions in vector output

### 🛠️ **Advanced Controls**
- **Curve Fitting Modes** - Pixel, polygon, or spline-based paths
- **Corner Detection** - Configurable angle thresholds
- **Color Difference Methods** - CIE Delta E and hybrid algorithms
- **Layer Growing** - Expand layer boundaries for better coverage

### 📊 **Quality Options**
- **Custom Color Palettes** - Specify exact colors for layer separation
- **Similarity Thresholds** - Control color grouping sensitivity
- **Chroma-Only Comparison** - Focus on hue rather than brightness
- **Hierarchical Clustering** - Stacked or cutout layer organization

## 📦 Installation

Add to your `Cargo.toml`:
```toml
vectorizer = { workspace = true }
```

## 🚀 Quick Start

### Basic Vectorization

```bash
# Simple cluster-based vectorization
vectorizer raster vectorize clusters -i input.png -o output.svg

# Layer-based vectorization (recommended for better quality)
vectorizer raster vectorize layers -i input.png -o output.svg
```

### Advanced Usage Examples

```bash
# High-quality vectorization with custom settings
vectorizer raster vectorize layers \
  -i photo.jpg \
  -o vector.svg \
  --color-precision 8 \
  --num-layers 10 \
  --mode spline \
  --remove-background

# Vectorize with specific colors
vectorizer raster vectorize layers \
  -i logo.png \
  -o logo.svg \
  --custom-colors "#FF0000,#00FF00,#0000FF" \
  --similarity 15 \
  --filter-speckle 8

# Fast processing with reduced quality
vectorizer raster vectorize clusters \
  -i sketch.png \
  -o sketch.svg \
  --mode polygon \
  --filter-speckle 2 \
  --color-precision 6
```

## 📖 CLI Reference

### Clusters Mode

Traditional clustering approach with VTracer enhancements:

```bash
vectorizer raster vectorize clusters [OPTIONS] --input <INPUT>
```

**Key Options:**
- `-i, --input` - Input raster image file
- `-o, --output` - Output SVG file  
- `-p, --color-precision` - Color comparison precision (1-8 bits) [default: 8]
- `-f, --filter-speckle` - Remove patches smaller than X pixels [default: 4]
- `-m, --mode` - Curve fitting: pixel, polygon, spline [default: spline]
- `--hierarchical` - Clustering mode: stacked, cutout [default: cutout]
- `--remove-background` - Automatically remove background

### Layers Mode (Recommended)

Advanced layer-based separation for superior quality results:

```bash
vectorizer raster vectorize layers [OPTIONS] --input <INPUT>
```

**Key Options:**
- `-i, --input` - Input raster image file
- `-o, --output` - Output SVG file
- `-l, --num-layers` - Number of color layers (auto-detected if omitted)
- `-c, --custom-colors` - Comma-separated hex colors (e.g., "#FF0000,#00FF00")
- `-s, --similarity` - Color similarity threshold
- `--color-difference` - Color comparison method: ciede, hybrid [default: ciede]
- `-g, --grow` - Expand layer boundaries by X pixels [default: 0]
- `--strict` - Only use pixels very similar to layer color
- `--only-chroma` - Compare only hue, ignore brightness

## 🎯 Algorithm Comparison

| Feature | Clusters Mode | Layers Mode |
|---------|---------------|-------------|
| **Quality** | Good | Excellent |
| **Speed** | Fast | Moderate |
| **Control** | Limited | Extensive |
| **Best For** | Simple images, logos | Complex images, photos |
| **Custom Colors** | No | Yes |
| **Layer Growing** | No | Yes |

## 📊 Quality Settings Guide

### For Logos and Simple Graphics
```bash
vectorizer raster vectorize layers \
  -i logo.png \
  -o logo.svg \
  --color-precision 8 \
  --mode spline \
  --filter-speckle 8
```

### For Photographs  
```bash
vectorizer raster vectorize layers \
  -i photo.jpg \
  -o photo.svg \
  --num-layers 15 \
  --similarity 20 \
  --grow 1 \
  --remove-background
```

### For Line Art and Sketches
```bash
vectorizer raster vectorize clusters \
  -i sketch.png \
  -o sketch.svg \
  --mode polygon \
  --color-precision 6 \
  --filter-speckle 2
```


## ⚙️ Technical Features

### Color Processing
- **CIE Delta E Color Difference** - Perceptually uniform color comparisons
- **Hybrid Color Algorithms** - Combine multiple color space calculations  
- **Chroma-Only Mode** - Focus on hue while ignoring brightness variations
- **Custom Color Precision** - 1-8 bit precision control for color grouping

### Path Generation
- **Spline Curves** - Smooth, mathematically precise curves
- **Polygon Mode** - Sharp edges for geometric designs
- **Pixel Mode** - Exact pixel replication for special cases
- **Corner Detection** - Automatic sharp corner identification

### Post-Processing
- **Speckle Filtering** - Remove noise and artifacts
- **Background Removal** - Smart background detection and removal
- **Layer Growing** - Expand thin lines and small features
- **Hierarchical Clustering** - Organize overlapping elements

## 🎨 Visual Comparison

The table below shows the difference in output quality between the two processing modes:

| Original | Clusters Mode | Layers Mode |
|:--------:|:------------:|:-----------:|
| <img src="article/assets/comparison/kos_input.png" width="300px"> | <img src="article/assets/comparison/clusters_kos_input.jpg" width="300px"> | <img src="article/assets/comparison/layers_kos_input.jpg" width="300px"> |
| <img src="article/assets/comparison/stars.jpg" width="300px"> | <img src="article/assets/comparison/clusters_stars.jpg" width="300px"> | <img src="article/assets/comparison/layers_stars.jpg" width="300px"> |
| <img src="article/assets/comparison/wolf.png" width="300px"> | <img src="article/assets/comparison/clusters_wolf.jpg" width="300px"> | <img src="article/assets/comparison/layers_wolf.jpg" width="300px"> |

## 🎯 Use Cases

- **Logo Vectorization** - Convert raster logos to scalable SVG format
- **Artwork Digitization** - Transform drawings and paintings into vector art
- **Icon Generation** - Create crisp icons from bitmap sources  
- **Print Preparation** - Generate scalable graphics for high-resolution printing
- **Web Optimization** - Reduce file sizes while maintaining quality
- **Design Workflows** - Bridge raster and vector design processes

## 🔧 Integration

The vectorizer can be used both as a command-line tool and integrated into Rust applications for programmatic image processing workflows.

## 📚 References

Built on top of [VTracer](https://github.com/visioncortex/vtracer) with significant enhancements for layer-based processing and improved quality control.
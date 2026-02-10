# Jewelry Assets Directory

## Overview
This directory contains 3D jewelry models in GLB (GL Transmission Format Binary) format.

## Responsibility Table

| File | Responsibility | Size | Type | Status |
|------|---------------|------|------|--------|
| 0.glb | Base jewelry model (legacy) | 3.3MB | 3D Model | Active |
| 1.glb | Jewelry model variant 1 (legacy) | 1.9MB | 3D Model | Active |
| 2.glb | Jewelry model variant 2 (optimized) | 401KB | 3D Model | Active |
| 3.glb | Jewelry model variant 3 | 3.4MB | 3D Model | Active |
| 4.glb | Jewelry model variant 4 (high-detail) | 12MB | 3D Model | Active |

## File Count Governance
- **Threshold**: 3+ files require readme.md documentation
- **Current Count**: 5 files
- **Status**: ✓ Compliant (readme.md present)

## Asset Size Analysis

### File Size Distribution
- **Total:** ~21MB (all 5 models)
- **Average:** ~4.2MB per model
- **Range:** 401KB - 12MB

### Large Asset: 4.glb (12MB)

**Status:** Unoptimized - Intentional high-detail variant

**Rationale:**
- Model 4 contains significantly more geometry/texture data than other variants
- 3.5x larger than the next largest file (3.glb at 3.4MB)
- Serves as high-fidelity showcase variant

**Optimization Options:**
1. **Texture Compression:** Convert embedded textures to WebP or Basis Universal
2. **Geometry Decimation:** Reduce polygon count for distant LOD
3. **Mesh Compression:** Apply Draco mesh compression
4. **Lazy Loading:** Model is already lazy-loaded on user request (implemented)

**Recommendation:**
- Keep current size as high detail is business requirement
- Monitor user feedback on load times

**Network Impact:**
- 4G connection: ~2-3 seconds to load
- 5G connection: ~1 second to load
- Loads only when user selects model 4 (not on initial page load)

### Optimized Asset: 2.glb (401KB)

**Status:** Well-optimized (10x smaller than average)

This model demonstrates effective optimization techniques and can serve as a reference for optimizing other models.

## Usage
These GLB files are used for 3D rendering and visualization of jewelry models within the jewelry configurator application.

Models are loaded lazily on-demand (see `configurator.rs:setup_rings()`) to optimize initial page load time.

## Maintenance
- Models should maintain consistent scale and orientation
- New models should follow the numeric naming convention (0-4)
- Update this readme when adding or removing models
- Consider optimization for models >5MB
- Test load times on mobile networks for large models

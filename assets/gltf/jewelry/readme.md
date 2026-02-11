# Jewelry Assets Directory

## Overview
This directory contains 3D jewelry models in GLB (GL Transmission Format Binary) format.

## Responsibility Table

| File | Responsibility | Size | Type | Status |
|------|---------------|------|------|--------|
| 0.glb | Base jewelry model (legacy) | 3.3MB | 3D Model | Active |
| 1.glb | Jewelry model variant 1 (legacy) | 1.9MB | 3D Model | Active |
| 2.glb | Jewelry model variant 2 (optimized) | 401KB | 3D Model | Active |

## File Count Governance
- **Threshold**: 3+ files require readme.md documentation
- **Current Count**: 3 files
- **Status**: ✓ Compliant (readme.md present)

## Asset Size Analysis

### File Size Distribution
- **Total:** ~5.6MB (all 3 models)
- **Average:** ~1.9MB per model
- **Range:** 401KB - 3.3MB

### Optimized Asset: 2.glb (401KB)

**Status:** Well-optimized (10x smaller than average)

This model demonstrates effective optimization techniques and can serve as a reference for optimizing other models.

## Usage
These GLB files are used for 3D rendering and visualization of jewelry models within the jewelry configurator application.

Models are loaded lazily on-demand (see `configurator.rs:setup_rings()`) to optimize initial page load time.

## Maintenance
- Models should maintain consistent scale and orientation
- New models should follow the numeric naming convention (0-2)
- Update this readme when adding or removing models
- Consider optimization for models >5MB
- Test load times on mobile networks for large models

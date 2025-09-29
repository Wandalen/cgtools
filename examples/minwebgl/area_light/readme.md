# PBR Area Lights with LTCs

![alt text](showcase.png)

## Physically-based shading with realistic, real-time rectangular area lights using Linearly Transformed Cosines (LTCs)

This demo showcases a cutting-edge, real-time rendering system that accurately simulates rectangular area light sources. It leverages the power of Physically-Based Rendering (PBR) and a novel technique called Linearly Transformed Cosines (LTCs) to produce realistic specular highlights that change shape based on the viewing angle.

## Features

### Superior Lighting Quality

- **Physically-Based Shading** - Uses the GGX microfacet model for realistic material appearance.
- **Rectangular Area Lights** - Moves beyond simplistic point lights to simulate realistic light sources with physical dimensions.
- **Soft Specular Highlights** - Specular reflections accurately stretch and change shape depending on the surface roughness and viewing angle.
- **Energy Conservation** - The lighting model correctly conserves energy, preventing objects from reflecting more light than they receive.

### Technical Advantages

- **Linearly Transformed Cosines (LTCs)** - A powerful technique that allows for the complex area light integral to be solved analytically and in real-time.
- **Precomputed Look-Up Tables (LUTs)** - Utilizes two small textures to store precomputed data for the LTC matrix and magnitude, enabling high performance on the GPU.
- **GPU Accelerated** - All complex lighting calculations are performed efficiently in the fragment shader.
- **Rust + WebAssembly** - Built with Rust and compiled to high-performance WebAssembly for native speed in the browser.

### Interactive Features

- **Dynamic Camera** - Orbit, pan, and zoom the camera to view the scene from any angle.
- **Transformable Light** - Move the area light within the scene to see the lighting and shadows update instantly.

## Quick Start

### Prerequisites

- WebGL 2.0 compatible browser
- Rust with `wasm32-unknown-unknown` target
- Trunk for development server

### Run the Example

```bash
# Navigate to the area light example
cd examples/minwebgl/area_light

# Install trunk if needed
cargo install trunk

# Serve the example
trunk serve --release
```

Open `http://localhost:8080` to see realistic area lights in action.

## 🔧 Technical Deep Dive

### The Challenge of Area Lights

Traditional real-time rendering uses point lights, which are a simplification. Real-world light sources have a physical shape, which results in soft shadows and varied specular highlights. Accurately calculating the contribution of every point on an area light's surface for every shaded pixel requires solving a complex integral, which is typically too slow for real-time applications.

### Linearly Transformed Cosines (LTCs)

This demo uses a groundbreaking technique called **Linearly Transformed Cosines (LTCs)** to solve this problem. The core idea is to approximate the complex Bidirectional Reflectance Distribution Function (BRDF) with a simpler distribution that can be analytically integrated.

1. **Simple Starting Point**: The process begins with a simple, easy-to-integrate clamped cosine distribution.
2. **Linear Transformation**: A 3x3 matrix is used to linearly transform this simple cosine shape, stretching and skewing it to closely match the appearance of a GGX microfacet BRDF lobe for a given material roughness and viewing angle.
3. **Analytic Integration**: Because of the properties of linear transformations, integrating the complex, transformed distribution over the area of the light is equivalent to integrating the simple cosine distribution over a transformed version of the light polygon. This new integral has a known analytical solution, which can be calculated very quickly on the GPU.

This entire process is made efficient by pre-calculating the transformation matrices for various roughness values and view angles and storing them in a small look-up texture.

## Lighting Comparison

| Method | Realism | Specular Shape | Performance |
| :--- | :--- | :--- | :--- |
| **Point Light** | Low | Uniform dot | Very High |
| **Monte Carlo (Stochastic)** | High | Accurate | Very Low |
| **Area Light (LTCs)** | High | Accurate & Dynamic | High |

## Tools and Resources

- [Real-Time Polygonal-Light Shading with Linearly Transformed Cosines](https://eheitzresearch.wordpress.com/415-2/) - The original SIGGRAPH 2016 paper by Eric Heitz, Jonathan Dupuy, Stephen Hill and David Neubelt.
- [LearnOpenGL - PBR Theory](https://learnopengl.com/PBR/Theory) - An excellent introduction to the theory behind Physically-Based Rendering.
- [LearnOpenGL - Area-Lights](https://learnopengl.com/Guest-Articles/2022/Area-Lights) - Pracrtical implementation of area-lights.

## Use Cases and Applications

### High-Fidelity Graphics

- **Game Development** - Enhancing realism for key light sources in game engines like Unity and Unreal.
- **Product Visualization** - Accurate representation of materials and lighting for e-commerce and marketing.
- **Architectural Rendering** - Simulating realistic interior and exterior lighting from windows and fixtures.

### Real-Time Visualization

- **Automotive Design** - Realistic paint and material shaders with accurate reflections.
- **Virtual and Augmented Reality** - Immersive experiences where lighting quality is critical for presence.
- **Scientific Visualization** - Accurately depicting how light interacts with different simulated materials.

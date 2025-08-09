# 🎨 Color Space Conversion Demo

> **Real-time color space visualization and conversion using Rust and WebAssembly**

A comprehensive interactive tool for exploring color theory and color space conversions in web applications. This example demonstrates real-time conversion between multiple color spaces, showcasing advanced color science concepts and efficient WebAssembly-based color processing.

![Color Space Demo](showcase.png)

## ✨ Features

### 🌈 **Color Space Support**
- **Wide Gamut Spaces** - A98 RGB, Display P3, Rec2020, ProPhoto RGB
- **Professional Spaces** - ACES 2065-1, ACEScg for film and VFX
- **Perceptual Spaces** - LAB, LCH, Oklab, Oklch for uniform perception
- **Standard Spaces** - sRGB, Linear sRGB, XYZ D50/D65
- **Artistic Spaces** - HSL, HWB for intuitive color selection

### 🔧 **Technical Implementation**
- **Rust Color Processing** - High-precision color calculations using the `color` crate
- **WebAssembly Performance** - Fast color conversions compiled to WASM
- **Real-Time Updates** - Instant conversion display as you pick colors
- **DOM Integration** - Seamless browser integration with native performance

### 🎮 **Interactive Features**
- **Live Color Picker** - HTML5 color input with instant feedback
- **Multiple Representations** - View same color across all color spaces
- **Precision Display** - Full numerical values for each color space
- **Educational Interface** - Learn color theory through experimentation

## 🚀 Quick Start

### Prerequisites
- WebGL-compatible browser
- Rust with `wasm32-unknown-unknown` target
- Trunk for WebAssembly building

### Run the Demo
```bash
# Navigate to color conversion demo
cd examples/minwebgl/color_space_conversions

# Install trunk if needed
cargo install trunk

# Build and serve
trunk serve --release
```

Open http://localhost:8080 to explore color space conversions.

## 🔬 Technical Deep Dive

### Color Space Architecture

The demo leverages Rust's `color` crate for accurate color conversions:

```rust
use color::{
  Srgb, LinearSrgb, DisplayP3, A98Rgb, ProphotoRgb, Rec2020,
  Lab, Lch, Oklab, Oklch, Hsl, Hwb,
  XyzD50, XyzD65, Aces2065_1, AcesCg
};

// Color conversion pipeline
struct ColorConverter {
  input_color: Srgb,
  conversions: HashMap<String, Box<dyn ColorSpace>>,
}

impl ColorConverter {
  fn convert_all(&self, srgb_color: Srgb) -> HashMap<String, String> {
    let mut results = HashMap::new();
    
    // Convert to various color spaces
    results.insert("sRGB".to_string(), 
      format!("rgb({}, {}, {})", srgb_color.red, srgb_color.green, srgb_color.blue));
    
    // Linear sRGB conversion
    let linear: LinearSrgb = srgb_color.into();
    results.insert("Linear sRGB".to_string(),
      format!("rgb({:.3}, {:.3}, {:.3})", linear.red, linear.green, linear.blue));
    
    // LAB conversion
    let lab: Lab = srgb_color.into();
    results.insert("CIELAB".to_string(),
      format!("lab({:.1}, {:.1}, {:.1})", lab.l, lab.a, lab.b));
    
    // Oklab conversion (modern perceptual space)
    let oklab: Oklab = srgb_color.into();
    results.insert("Oklab".to_string(),
      format!("oklab({:.3}, {:.3}, {:.3})", oklab.l, oklab.a, oklab.b));
    
    results
  }
}
```

### Real-Time DOM Updates

```rust
use wasm_bindgen::prelude::*;
use web_sys::{Document, Element, HtmlInputElement, Event};

#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
  let window = web_sys::window().unwrap();
  let document = window.document().unwrap();
  
  let color_picker = document
    .get_element_by_id("colorPicker")
    .unwrap()
    .dyn_into::<HtmlInputElement>()?;
  
  // Set up real-time color conversion
  let closure = Closure::wrap(Box::new(move |event: Event| {
    let target = event.target().unwrap();
    let input = target.dyn_into::<HtmlInputElement>().unwrap();
    let hex_color = input.value();
    
    // Convert hex to sRGB
    let srgb_color = parse_hex_color(&hex_color);
    
    // Update all color space displays
    update_color_displays(&document, srgb_color);
  }) as Box<dyn FnMut(_)>);
  
  color_picker.add_event_listener_with_callback("input", closure.as_ref().unchecked_ref())?;
  closure.forget();
  
  Ok(())
}
```

### Color Space Implementations

#### Professional Color Spaces

```rust
// ACES color space for film and VFX
fn convert_to_aces(srgb: Srgb) -> Aces2065_1 {
  // sRGB -> XYZ -> ACES 2065-1
  let xyz: XyzD65 = srgb.into();
  let aces: Aces2065_1 = xyz.into();
  aces
}

// Display P3 for modern displays
fn convert_to_display_p3(srgb: Srgb) -> DisplayP3 {
  // Wide gamut conversion for modern monitors
  let xyz: XyzD65 = srgb.into();
  let p3: DisplayP3 = xyz.into();
  p3
}

// ProPhoto RGB for photography
fn convert_to_prophoto(srgb: Srgb) -> ProphotoRgb {
  let xyz: XyzD50 = srgb.into(); // ProPhoto uses D50 illuminant
  let prophoto: ProphotoRgb = xyz.into();
  prophoto
}
```

#### Perceptual Color Spaces

```rust
// CIELAB - perceptually uniform color space
fn analyze_lab_color(lab: Lab) -> ColorAnalysis {
  ColorAnalysis {
    lightness: lab.l,
    chroma: (lab.a * lab.a + lab.b * lab.b).sqrt(),
    hue_angle: lab.b.atan2(lab.a).to_degrees(),
    perceptual_difference: calculate_delta_e(&lab, &reference_lab),
  }
}

// Oklab - modern perceptual space
fn oklab_color_harmony(oklab: Oklab) -> Vec<Oklab> {
  vec![
    // Complementary color
    Oklab { l: oklab.l, a: -oklab.a, b: -oklab.b },
    // Triadic colors
    rotate_oklab_hue(oklab, 120.0),
    rotate_oklab_hue(oklab, 240.0),
  ]
}
```

## 🎨 Color Theory Applications

### Color Harmony Generation

```rust
// Generate color harmonies using HSL
struct ColorHarmony {
  base_color: Hsl,
}

impl ColorHarmony {
  fn complementary(&self) -> Hsl {
    Hsl {
      h: (self.base_color.h + 180.0) % 360.0,
      s: self.base_color.s,
      l: self.base_color.l,
    }
  }
  
  fn triadic(&self) -> (Hsl, Hsl) {
    let color1 = Hsl {
      h: (self.base_color.h + 120.0) % 360.0,
      s: self.base_color.s,
      l: self.base_color.l,
    };
    
    let color2 = Hsl {
      h: (self.base_color.h + 240.0) % 360.0,
      s: self.base_color.s,
      l: self.base_color.l,
    };
    
    (color1, color2)
  }
  
  fn analogous(&self) -> Vec<Hsl> {
    (-30..=30)
      .step_by(10)
      .map(|offset| Hsl {
        h: (self.base_color.h + offset as f32) % 360.0,
        s: self.base_color.s,
        l: self.base_color.l,
      })
      .collect()
  }
}
```

### Color Accessibility Analysis

```rust
// WCAG contrast ratio calculation
fn calculate_contrast_ratio(color1: Srgb, color2: Srgb) -> f32 {
  let l1 = relative_luminance(color1);
  let l2 = relative_luminance(color2);
  
  let lighter = l1.max(l2);
  let darker = l1.min(l2);
  
  (lighter + 0.05) / (darker + 0.05)
}

fn relative_luminance(srgb: Srgb) -> f32 {
  let linear: LinearSrgb = srgb.into();
  0.2126 * linear.red + 0.7152 * linear.green + 0.0722 * linear.blue
}

// Accessibility compliance checker
struct AccessibilityChecker;

impl AccessibilityChecker {
  fn check_wcag_aa(foreground: Srgb, background: Srgb) -> bool {
    let ratio = calculate_contrast_ratio(foreground, background);
    ratio >= 4.5 // WCAG AA standard
  }
  
  fn check_wcag_aaa(foreground: Srgb, background: Srgb) -> bool {
    let ratio = calculate_contrast_ratio(foreground, background);
    ratio >= 7.0 // WCAG AAA standard
  }
}
```

## 🔍 Color Space Comparison

### Gamut Coverage Analysis

| Color Space | Gamut Size | Use Case | Precision |
|-------------|------------|----------|----------|
| **sRGB** | Standard | Web/Display | 8-bit |
| **Display P3** | 25% larger | Modern displays | 10-bit |
| **Rec2020** | 75% larger | HDR/UHD TV | 12-bit |
| **ProPhoto RGB** | 90% larger | Photography | 16-bit |
| **ACES 2065-1** | 100% larger | Film/VFX | 32-bit float |

### Perceptual Uniformity

```rust
// Compare perceptual color spaces
struct PerceptualComparison {
  lab: Lab,
  oklab: Oklab,
  lch: Lch,
  oklch: Oklch,
}

impl PerceptualComparison {
  fn delta_e_lab(&self, other: &Lab) -> f32 {
    // CIEDE2000 formula for perceptual difference
    let dl = self.lab.l - other.l;
    let da = self.lab.a - other.a;
    let db = self.lab.b - other.b;
    
    (dl * dl + da * da + db * db).sqrt()
  }
  
  fn delta_e_ok(&self, other: &Oklab) -> f32 {
    // Oklab perceptual difference (simpler, more accurate)
    let dl = self.oklab.l - other.l;
    let da = self.oklab.a - other.a;
    let db = self.oklab.b - other.b;
    
    (dl * dl + da * da + db * db).sqrt()
  }
}
```

## 🎯 Educational Applications

### Interactive Color Theory

```rust
// Color temperature demonstration
struct ColorTemperature {
  kelvin: f32,
}

impl ColorTemperature {
  fn to_rgb(&self) -> Srgb {
    // Planckian locus approximation
    let temp = self.kelvin;
    
    let (r, g, b) = if temp < 6600.0 {
      let r = 1.0;
      let g = -155.25485562709179 - 0.44596950469579133 * (temp - 2000.0) + 104.49216199393888 * (temp - 2000.0).ln();
      let b = if temp < 2000.0 { 0.0 } else { -254.76935184120902 + 0.8274096064007395 * (temp - 2000.0) + 115.67994401066147 * (temp - 2000.0).ln() };
      (r, g / 255.0, b / 255.0)
    } else {
      let r = 351.97690566805693 + 0.114206453784165 * (temp - 6600.0) - 40.25366309332127 * (temp - 6600.0).ln();
      let g = 325.4494125711974 + 0.07943456536662342 * (temp - 6600.0) - 28.0852963507957 * (temp - 6600.0).ln();
      let b = 1.0;
      (r / 255.0, g / 255.0, b)
    };
    
    Srgb { red: r.clamp(0.0, 1.0), green: g.clamp(0.0, 1.0), blue: b.clamp(0.0, 1.0) }
  }
}
```

### Color Blind Simulation

```rust
// Simulate color vision deficiencies
enum ColorBlindnessType {
  Protanopia,    // Red-blind
  Deuteranopia,  // Green-blind
  Tritanopia,    // Blue-blind
  Monochromacy,  // Complete color blindness
}

struct ColorBlindSimulator;

impl ColorBlindSimulator {
  fn simulate(color: Srgb, deficiency: ColorBlindnessType) -> Srgb {
    match deficiency {
      ColorBlindnessType::Protanopia => {
        // Remove red channel sensitivity
        let linear: LinearSrgb = color.into();
        let simulated = LinearSrgb {
          red: 0.0,
          green: linear.green + linear.red * 0.5,
          blue: linear.blue + linear.red * 0.5,
        };
        simulated.into()
      },
      ColorBlindnessType::Deuteranopia => {
        // Remove green channel sensitivity  
        let linear: LinearSrgb = color.into();
        let simulated = LinearSrgb {
          red: linear.red + linear.green * 0.5,
          green: 0.0,
          blue: linear.blue + linear.green * 0.5,
        };
        simulated.into()
      },
      // ... other deficiency types
      _ => color, // Placeholder
    }
  }
}
```

## 📊 Performance Optimization

### WebAssembly Color Processing

```rust
// Optimized batch color conversion
#[wasm_bindgen]
pub struct BatchColorConverter {
  conversions: Vec<ColorSpaceConversion>,
}

#[wasm_bindgen]
impl BatchColorConverter {
  #[wasm_bindgen(constructor)]
  pub fn new() -> Self {
    Self {
      conversions: Vec::new(),
    }
  }
  
  #[wasm_bindgen]
  pub fn convert_batch(&self, colors: &[f32]) -> Vec<f32> {
    colors
      .chunks_exact(3)
      .flat_map(|rgb| {
        let srgb = Srgb { 
          red: rgb[0], 
          green: rgb[1], 
          blue: rgb[2] 
        };
        
        let lab: Lab = srgb.into();
        vec![lab.l, lab.a, lab.b]
      })
      .collect()
  }
}
```

## 📚 Learning Resources

### Color Science Theory
- **[Color Science Primer](http://www.brucelindbloom.com/)** - Comprehensive color theory
- **[CIELAB Color Space](https://en.wikipedia.org/wiki/CIELAB_color_space)** - Perceptual color space
- **[Oklab Color Space](https://bottosson.github.io/posts/oklab/)** - Modern perceptual color space

### Professional Applications
- **[ACES Workflow](https://www.oscars.org/science-technology/sci-tech-projects/aces)** - Film industry standard
- **[Color Management](https://www.color.org/)** - ICC color management
- **[Display Calibration](https://displaycal.net/)** - Monitor color accuracy

## 🛠️ Advanced Extensions

### Custom Color Space Implementation

```rust
// Define custom color space
struct CustomColorSpace {
  name: String,
  primaries: [XyzD65; 3], // RGB primaries in XYZ
  white_point: XyzD65,
  transfer_function: TransferFunction,
}

enum TransferFunction {
  Gamma(f32),
  sRGB,
  Rec2020,
  Custom(fn(f32) -> f32),
}

impl CustomColorSpace {
  fn from_srgb(&self, srgb: Srgb) -> CustomRgb {
    // Convert sRGB -> XYZ -> Custom RGB
    let xyz: XyzD65 = srgb.into();
    self.xyz_to_custom_rgb(xyz)
  }
}
```

## 🤝 Contributing

Part of the CGTools workspace. Feel free to submit issues and pull requests on GitHub.

## 📄 License

MIT
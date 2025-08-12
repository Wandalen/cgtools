# 🎬 Video as Texture

> **Real-time video texturing with WebGL 2.0 for dynamic multimedia rendering**

A comprehensive demonstration of using video files as dynamic textures in WebGL applications. This example showcases advanced texture techniques including video streaming, automatic playback, and GPU-accelerated rendering for creating interactive multimedia experiences.

![Video Texture Demo](./showcase.gif)

## ✨ Features

### 🎥 **Video Processing**
- **Dynamic Textures** - Real-time video streaming to GPU textures
- **Format Support** - MP4, WebM, and other browser-supported formats
- **Auto Playback** - Seamless looping and playback control
- **Resolution Flexibility** - Support for various video dimensions

### 🔧 **Technical Implementation**
- **WebGL 2.0 Optimized** - Hardware-accelerated video rendering
- **Texture Streaming** - Efficient GPU memory usage for video data
- **Frame Synchronization** - Smooth video-to-texture updates
- **Cross-Platform** - Works across modern browsers and devices

### 🎮 **Interactive Features**
- **Real-Time Updates** - Live video texture updates during playback
- **Custom Video Support** - Easy integration of user-provided videos
- **Performance Monitoring** - Optimized for smooth playback

## 🚀 Quick Start

### Prerequisites
- WebGL 2.0 compatible browser
- Rust with `wasm32-unknown-unknown` target
- Trunk for development server

### Run the Example
```bash
# Navigate to video texture example
cd examples/minwebgl/video_as_texture

# Install trunk if needed
cargo install trunk

# Serve the example
trunk serve --release
```

Open http://localhost:8080 to see video texturing in action.

## 🔧 Technical Deep Dive

### Video-to-Texture Pipeline

Modern video texturing leverages HTML5 video elements with WebGL integration:

```rust
// Video element creation and setup
fn setup_video_element(video_path: &str) -> Result<HtmlVideoElement, JsValue> {
  let window = web_sys::window().unwrap();
  let document = window.document().unwrap();
  
  let video: HtmlVideoElement = document
    .create_element("video")?
    .dyn_into::<HtmlVideoElement>()?;
  
  // Configure video properties
  video.set_src(video_path);
  video.set_loop(true);
  video.set_muted(true);
  video.set_autoplay(true);
  video.set_cross_origin(Some("anonymous"));
  
  Ok(video)
}
```

### WebGL Texture Creation

```rust
// Create and configure video texture
fn create_video_texture(
  gl: &WebGl2RenderingContext,
  video: &HtmlVideoElement
) -> Result<WebGlTexture, JsValue> {
  let texture = gl.create_texture().unwrap();
  
  gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&texture));
  
  // Configure texture parameters for video
  gl.tex_parameteri(
    WebGl2RenderingContext::TEXTURE_2D,
    WebGl2RenderingContext::TEXTURE_WRAP_S,
    WebGl2RenderingContext::CLAMP_TO_EDGE as i32,
  );
  gl.tex_parameteri(
    WebGl2RenderingContext::TEXTURE_2D,
    WebGl2RenderingContext::TEXTURE_WRAP_T,
    WebGl2RenderingContext::CLAMP_TO_EDGE as i32,
  );
  gl.tex_parameteri(
    WebGl2RenderingContext::TEXTURE_2D,
    WebGl2RenderingContext::TEXTURE_MIN_FILTER,
    WebGl2RenderingContext::LINEAR as i32,
  );
  gl.tex_parameteri(
    WebGl2RenderingContext::TEXTURE_2D,
    WebGl2RenderingContext::TEXTURE_MAG_FILTER,
    WebGl2RenderingContext::LINEAR as i32,
  );
  
  Ok(texture)
}
```

### Real-Time Texture Updates

```rust
// Update video texture each frame
fn update_video_texture(
  gl: &WebGl2RenderingContext,
  texture: &WebGlTexture,
  video: &HtmlVideoElement
) -> Result<(), JsValue> {
  gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(texture));
  
  // Upload current video frame to texture
  gl.tex_image_2d_with_u32_and_u32_and_html_video_element(
    WebGl2RenderingContext::TEXTURE_2D,
    0, // mip level
    WebGl2RenderingContext::RGB as i32,
    WebGl2RenderingContext::RGB,
    WebGl2RenderingContext::UNSIGNED_BYTE,
    video,
  )?;
  
  Ok(())
}
```

### Video Shader Implementation

```glsl
// Vertex shader for video quad
#version 300 es
precision mediump float;

in vec2 position;
in vec2 texCoord;

uniform mat4 mvpMatrix;

out vec2 vTexCoord;

void main() {
  gl_Position = mvpMatrix * vec4(position, 0.0, 1.0);
  vTexCoord = texCoord;
}
```

```glsl
// Fragment shader for video rendering
#version 300 es
precision mediump float;

in vec2 vTexCoord;

uniform sampler2D videoTexture;
uniform float time;
uniform float brightness;
uniform float contrast;

out vec4 fragColor;

void main() {
  vec4 videoColor = texture(videoTexture, vTexCoord);
  
  // Apply video effects
  videoColor.rgb *= brightness;
  videoColor.rgb = (videoColor.rgb - 0.5) * contrast + 0.5;
  
  fragColor = videoColor;
}
```

## 🎥 Customization Guide

### Using Your Own Videos

1. **Add Video File**
   ```bash
   # Place your video in the static folder
   cp your_video.mp4 static/
   ```

2. **Update Configuration**
   ```rust
   // Modify these constants in main.rs
   const VIDEO_PATH: &str = "static/your_video.mp4";
   const VIDEO_WIDTH: u32 = 1920;  // Your video width
   const VIDEO_HEIGHT: u32 = 1080; // Your video height
   ```

3. **Supported Formats**
   - **MP4** - H.264/H.265 encoding (recommended)
   - **WebM** - VP8/VP9 encoding
   - **OGV** - Theora encoding (legacy support)

### Video Optimization Tips

```rust
// Optimal video configuration
struct VideoConfig {
  // Keep reasonable resolution for smooth playback
  max_width: u32,     // 1920 or lower for web
  max_height: u32,    // 1080 or lower for web
  
  // Optimize bitrate for web delivery
  target_bitrate: u32, // 2-5 Mbps for web
  
  // Use web-optimized codecs
  codec: VideoCodec,   // H.264 baseline profile
}

impl VideoConfig {
  fn web_optimized() -> Self {
    Self {
      max_width: 1280,
      max_height: 720,
      target_bitrate: 3_000_000, // 3 Mbps
      codec: VideoCodec::H264Baseline,
    }
  }
}
```

## 📊 Performance Considerations

### Video Texture Performance

| Video Resolution | Memory Usage | Performance | Recommendation |
|-----------------|---------------|-------------|----------------|
| **720p (1280x720)** | ~3.7MB | Excellent | Recommended |
| **1080p (1920x1080)** | ~8.3MB | Good | High-end devices |
| **4K (3840x2160)** | ~33MB | Poor | Not recommended |

### Optimization Strategies

```rust
// Frame rate limiting for better performance
struct VideoRenderer {
  last_update: f64,
  target_fps: f32,
  skip_frames: bool,
}

impl VideoRenderer {
  fn should_update_texture(&mut self, current_time: f64) -> bool {
    let frame_interval = 1000.0 / self.target_fps as f64;
    
    if current_time - self.last_update >= frame_interval {
      self.last_update = current_time;
      true
    } else {
      false
    }
  }
  
  fn render_frame(&mut self, video: &HtmlVideoElement) {
    // Only update texture when needed
    if self.should_update_texture(performance::now()) {
      self.update_video_texture(video);
    }
    
    self.render_quad();
  }
}
```

## 🎯 Advanced Features

### Multi-Video Texturing

```rust
// Handle multiple video sources
struct MultiVideoRenderer {
  videos: Vec<HtmlVideoElement>,
  textures: Vec<WebGlTexture>,
  current_video: usize,
}

impl MultiVideoRenderer {
  fn switch_video(&mut self, index: usize) {
    if index < self.videos.len() {
      self.current_video = index;
    }
  }
  
  fn render_current(&self) {
    let video = &self.videos[self.current_video];
    let texture = &self.textures[self.current_video];
    
    self.update_texture(texture, video);
    self.render_with_texture(texture);
  }
}
```

### Video Effects Pipeline

```glsl
// Advanced video effects in fragment shader
#version 300 es
precision mediump float;

in vec2 vTexCoord;

uniform sampler2D videoTexture;
uniform float time;
uniform float chromaticAberration;
uniform float scanlineIntensity;
uniform float noiseLevel;

out vec4 fragColor;

// Chromatic aberration effect
vec3 chromaticAberrationEffect(vec2 uv, float strength) {
  vec2 distortion = (uv - 0.5) * strength;
  
  float r = texture(videoTexture, uv + distortion).r;
  float g = texture(videoTexture, uv).g;
  float b = texture(videoTexture, uv - distortion).b;
  
  return vec3(r, g, b);
}

// CRT scanline effect
float scanlines(vec2 uv, float intensity) {
  return 1.0 - intensity * sin(uv.y * 800.0) * 0.5;
}

// Film noise effect
float noise(vec2 uv, float time) {
  return fract(sin(dot(uv + time, vec2(12.9898, 78.233))) * 43758.5453);
}

void main() {
  vec2 uv = vTexCoord;
  
  // Apply chromatic aberration
  vec3 color = chromaticAberrationEffect(uv, chromaticAberration);
  
  // Add scanlines
  color *= scanlines(uv, scanlineIntensity);
  
  // Add film noise
  float filmNoise = noise(uv, time) * noiseLevel;
  color += filmNoise;
  
  fragColor = vec4(color, 1.0);
}
```

## 🎮 Use Cases

### Game Development
- **Animated Billboards** - Video advertisements in 3D environments
- **Cutscene Integration** - Seamless video playback within game worlds
- **UI Elements** - Dynamic video backgrounds for menus
- **Environmental Effects** - Animated water, fire, or weather systems

### Interactive Media
- **Digital Signage** - Dynamic advertising displays
- **Art Installations** - Interactive video sculptures
- **Educational Content** - Video-enhanced learning materials
- **Virtual Environments** - Immersive video experiences

## 📚 Learning Resources

### Video Processing
- **[MDN Video API](https://developer.mozilla.org/en-US/docs/Web/API/HTMLVideoElement)** - Complete video element reference
- **[WebGL Video](https://webglfundamentals.org/webgl/lessons/webgl-video.html)** - Video texturing fundamentals
- **[Video Optimization](https://web.dev/video/)** - Web video best practices

### Advanced Techniques
- **[Video Effects](https://www.shadertoy.com/results?query=video)** - Shader-based video effects
- **[WebGL Performance](https://developer.mozilla.org/en-US/docs/Web/API/WebGL_API/WebGL_best_practices)** - Optimization guidelines
- **[HTML5 Video](https://html.spec.whatwg.org/multipage/media.html)** - Video specification

## 🛠️ Troubleshooting

### Common Issues
- **CORS Errors** - Ensure proper cross-origin headers for video files
- **Codec Support** - Use widely supported formats (H.264/MP4)
- **Performance** - Consider video resolution and bitrate optimization
- **Mobile Compatibility** - Test autoplay policies on mobile devices

### Debug Techniques
```rust
// Video debugging utilities
fn debug_video_state(video: &HtmlVideoElement) {
  console::log_1(&format!(
    "Video State - Ready: {}, Current Time: {:.2}, Duration: {:.2}",
    video.ready_state(),
    video.current_time(),
    video.duration()
  ).into());
}

// Performance monitoring
fn monitor_texture_updates() {
  let start = performance::now();
  // ... texture update code ...
  let end = performance::now();
  
  if end - start > 16.0 { // >16ms indicates dropped frames
    console::warn_1(&"Slow video texture update detected".into());
  }
}
```


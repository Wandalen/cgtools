//! Type definitions for the deferred shading example

use minwebgl as gl;
use std::{ cell::RefCell, rc::Rc };
use web_sys::{ WebGlFramebuffer, WebGlRenderbuffer, WebGlTexture };
use crate::elliptical_orbit::EllipticalOrbit;

/// Parameters for the GUI controls
#[ derive( Debug, serde::Serialize, serde::Deserialize ) ]
pub struct GuiParams
{
  /// Number of active lights to render
  pub light_count : usize,
  /// RGB color of all lights
  pub light_color : [ f32; 3 ],
  /// Minimum radius for randomly generated light volumes
  pub min_radius : f32,
  /// Maximum radius for randomly generated light volumes
  pub max_radius : f32,
  /// Intensity multiplier for all lights
  pub intensity : f32,
}

/// Shader programs used for rendering
pub struct Shaders
{
  pub light : gl::shader::Program,
  pub object : gl::shader::Program,
  pub screen : gl::shader::Program,
  pub light_sphere : gl::shader::Program,
}

/// Framebuffers and render targets
pub struct Framebuffers
{
  pub gbuffer : Option< WebGlFramebuffer >,
  pub position_gbuffer : Option< WebGlTexture >,
  pub normal_gbuffer : Option< WebGlTexture >,
  pub color_gbuffer : Option< WebGlTexture >,
  pub depthbuffer : Option< WebGlRenderbuffer >,
  pub offscreen_buffer : Option< WebGlFramebuffer >,
  pub offscreen_color : Option< WebGlTexture >,
}

/// Light system data
#[ allow( dead_code ) ]
pub struct LightSystem
{
  pub translations : Vec< [ f32; 3 ] >,
  pub translation_buffer : gl::WebGlBuffer,
  pub radii : Rc< RefCell< Vec< f32 > > >,
  pub radius_buffer : gl::WebGlBuffer,
  pub orbits : Vec< EllipticalOrbit >,
  pub offsets : Vec< f32 >,
  pub prev_radius_range : Rc< RefCell< ( f32, f32 ) > >,
  pub max_count : usize,
}

/// Geometry for rendering
pub struct RenderGeometry
{
  pub light_volume : renderer::webgl::Geometry,
  pub light_sphere : Rc< RefCell< renderer::webgl::Geometry > >,
}

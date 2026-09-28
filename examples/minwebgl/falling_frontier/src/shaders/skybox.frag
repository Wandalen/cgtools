#version 300 es
precision highp float;

// Runtime half of the baked nebula skybox - `background.rs`'s `bake_cubemap`
// evaluates `background.frag`'s procedural fbm formula once per cube face at
// startup; this shader just reconstructs the per-pixel ray direction (same
// math as `background.frag`) and samples the result, so the expensive noise
// never runs per-frame.

in vec2 v_ndc;

uniform mat4 u_inv_view_proj;
uniform vec3 u_camera_position;
uniform samplerCube u_skybox;

out vec4 frag_color;

void main()
{
  vec4 far = u_inv_view_proj * vec4( v_ndc, 1.0, 1.0 );
  far.xyz /= far.w;
  vec3 ray_dir = normalize( far.xyz - u_camera_position );

  vec3 color = texture( u_skybox, ray_dir ).rgb;

  // Soft falloff right at the bottom edge only, echoing the reference
  // screenshot's letterbox vignette without darkening the rest of the sky -
  // kept here (screen-space, the real camera's `v_ndc`) rather than baked
  // into the cube map, which would fix a bake-camera screen edge as a
  // permanent seam in world space (see `background.frag`'s own note).
  float vignette = 1.0 - 0.25 * smoothstep( 0.55, 1.0, -v_ndc.y );
  color *= vignette;

  frag_color = vec4( color, 1.0 );
}

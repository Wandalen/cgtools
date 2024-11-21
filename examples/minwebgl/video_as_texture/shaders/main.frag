#version 300 es

// Set default precision for float calculations
precision mediump float;

// Texture sampler uniform
uniform sampler2D u_sampler;

// Interpolated texture coordinates from vertex shader
in vec2 v_tex_coord;

// Output color for this fragment
out vec4 frag_color;

void main()
{
  // Sample the texture at the interpolated texture coordinates
  frag_color = texture( u_sampler, v_tex_coord );
}

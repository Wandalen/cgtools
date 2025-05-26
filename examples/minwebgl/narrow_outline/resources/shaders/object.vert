#version 300 es
// Input vertex position attribute.
layout ( location = 0 ) in vec3 a_pos;

// Uniform transformation matrices: Projection, View, and Model.
// These are uploaded from the Rust application code.
uniform mat4 u_projection; // Camera projection matrix
uniform mat4 u_view;       // Camera view matrix
uniform mat4 u_model;      // Object's model matrix ( applied to the entire model )

void main()
{
  // Transform the input vertex position from model space to clip space
  // by applying the model, view, and projection matrices in order.
  gl_Position = u_projection * u_view * u_model * vec4( a_pos, 1.0 );
}
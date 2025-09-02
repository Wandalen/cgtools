#version 300 es
// Input vertex position attribute.
layout ( location = 0 ) in vec3 a_pos;
// Input normal attribute.
layout ( location = 1 ) in vec3 a_norm;
// Input object id attribute.
layout ( location = 2 ) in float a_object_id;

// Output varying for the fragment shader (transformed normal).
out vec3 v_norm;
// Output flat for the fragment shader.
out float v_object_id;

// Uniform transformation matrices: Projection, View, and Model.
// These are uploaded from the Rust application code.
uniform mat4 u_projection; // Camera projection matrix
uniform mat4 u_view;       // Camera view matrix
uniform mat4 u_model;      // Object's model matrix ( applied to the entire model )

// Uniform for the normal matrix.
uniform mat4 u_normal_matrix;

void main()
{
  // Transform the input vertex position from model space to clip space
  // by applying the model, view, and projection matrices in order.
  gl_Position = u_projection * u_view * u_model * vec4( a_pos, 1.0 );

  // Transform the normal vector to view space using the normal matrix.
  // We only care about the direction, so we set w to 0.
  v_norm = mat3( u_normal_matrix ) * a_norm;
  v_object_id = a_object_id;
}
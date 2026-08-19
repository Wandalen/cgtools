#version 300 es

layout ( location = 0 ) in vec3 a_position;
layout ( location = 1 ) in vec3 a_color;

uniform mat4 u_view_proj;

out vec3 v_color;

void main()
{
  v_color = a_color;
  gl_Position = u_view_proj * vec4( a_position, 1.0 );

  // three.js's PointsMaterial({ sizeAttenuation: true }) shrinks points with
  // distance (mirroring its own `gl_PointSize *= scale / -mvPosition.z`) -
  // without it every star reads at the same size/prominence regardless of
  // depth, which visually exaggerates the perspective clustering inherent
  // to a box this large (far more points lie far from the camera than
  // near it, and they all converge toward the same screen-space region).
  // `gl_Position.w` is exactly `-view_z` for this projection, so this is
  // the same ratio three.js computes.
  gl_PointSize = clamp( 700.0 / gl_Position.w, 1.0, 4.0 );
}

#version 300 es

// Fullscreen triangle vertex shader
// Generates positions without vertex buffer

out vec2 v_uv;

void main()
{
  // Generate fullscreen triangle using vertex ID
  // Triangle covers entire NDC space [-1, 1]
  float x = float( ( gl_VertexID & 1 ) << 2 );
  float y = float( ( gl_VertexID & 2 ) << 1 );

  v_uv = vec2( x, y ) * 0.5;
  gl_Position = vec4( x - 1.0, y - 1.0, 0.0, 1.0 );
}

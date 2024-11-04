#version 300 es

layout( location = 0 ) in float scale_y;
layout( location = 1 ) in float scale_x;
layout( location = 2 ) in float horizontal_offset;
layout( location = 3 ) in vec3 color;

out vec3 v_color;

void main()
{
  // these are vertices of a quad meant to be
  // rendered with GL::TRIANGLE_STRIP call
  const vec2 VERTICES[] = vec2[]
  (
    vec2( 0.0, -1.0 ), vec2( 1.0, -1.0 ),
    vec2( 0.0,  1.0 ), vec2( 1.0,  1.0 )
  );

  vec2 position = VERTICES[ gl_VertexID ];
  position.y *= scale_y;
  position.x *= scale_x;
  position.x += scale_x * horizontal_offset;
  position.x = 1.0 - position.x;
  v_color = color;

  gl_Position = vec4( position, 0.0, 1.0 );
}

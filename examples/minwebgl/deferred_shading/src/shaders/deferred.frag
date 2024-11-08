#version 300 es
precision mediump float;

struct PointLight
{
  highp vec3 position;
  highp vec3 color;
};

in vec2 v_texcoord;

uniform sampler2D positions;
uniform sampler2D normals;
layout( std140 ) uniform Lights
{
  PointLight lights[ 50 ];
};

out vec4 frag_color;

void main()
{
  vec3 color = vec3( 1.0, 1.0, 0.0 );
  for ( int i = 0; i < 50; i++ )
  {
    PointLight light = lights[ i ];

  }
  vec3 normal = texture( normals, v_texcoord ).xyz;
  vec3 direction = vec3( 0.0, 0.0, -1.0 );
  float color = dot( normal, -direction );
  frag_color = vec4( color * vec3( 1.0 ), 1.0 );
}

#version 300 es
#define NUM_LIGHTS 50
precision mediump float;

struct PointLight
{
  highp vec4 position;
  highp vec4 color;
};

in vec2 v_texcoord;

uniform sampler2D positions;
uniform sampler2D normals;
layout( std140 ) uniform Lights
{
  PointLight lights[ NUM_LIGHTS ];
};

out vec4 frag_color;

void main()
{
  const vec3 COLOR = vec3( 1.0 );

  vec3 position = texture( positions, v_texcoord ).xyz;
  vec3 normal = texture( normals, v_texcoord ).xyz;
  vec3 illumination = vec3( 0.0 );

  for ( int i = 0; i < lights.length(); i++ )
  {
    PointLight light = lights[ i ];
    vec3 offset = light.position.xyz - position;
    vec3 direction = normalize( offset );
    float len = length( offset );
    float attenuation = 1.0 / ( len * len );
    illumination += COLOR * light.color.rgb * max( dot( normal, direction ), 0.0 ) * attenuation;
  }

  frag_color = vec4( illumination, 1.0 );
}

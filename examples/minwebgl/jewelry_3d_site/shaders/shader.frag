#version 300 es

precision mediump float;

uniform samplerCube cube_map;
uniform float max_distance;

in vec3 vNormal;
in vec3 vPosition;

out vec4 frag_color;

// Extract the data from the normal map
vec4 getNormalData( vec3 dir )
{
  vec4 data = texture( cube_map, dir );
  data.rgb = normalize( data.rgb * 2.0 - 1.0 );
  return data;
}

void main()
{
  frag_color = getNormalData( vPosition );
}


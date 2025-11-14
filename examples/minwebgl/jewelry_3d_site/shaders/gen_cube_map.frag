#version 300 es

precision mediump float;

uniform float maxDistance;

in vec3 vNormal;
in vec3 vPosition;

out vec4 frag_color;

void main()
{
  vec3 normal = vNormal * 0.5 + 0.5;
  // We store normal in range from 0.0 to 1.0 in rgb channels
  // And store the normalized distance from the origin to the surface in alpha channel
  frag_color = vec4( normal, length( vPosition ) / maxDistance );
}


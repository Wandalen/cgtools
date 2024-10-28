#version 300 es

precision mediump float;

in vec3 vNormal;
in vec2 vUv;

out vec4 frag_color;

void main()
{
  vec3 lightDir = vec3( 1.0, 1.0, 1.0 );
  vec3 ambientLight = vec3( 1.0 ) * 0.1;
  vec3 directionalLight = vec3( 1.0 );

  vec3 diffuseColor = vec3( 1.0, 0.0, 0.0 );
  float diffuseValue = clamp( dot( vNormal, lightDir ), 0.0, 1.0 );

  vec3 color = diffuseColor * ( ambientLight + directionalLight * diffuseValue );

  // Gamma correciton
  color = pow( color, vec3( 1.0 / 2.2 ) );
  frag_color = vec4( color, 1.0 );
}

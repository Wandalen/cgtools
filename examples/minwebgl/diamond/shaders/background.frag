#version 300 es

precision mediump float;

uniform mat4x4 viewMatrix;
uniform samplerCube envMap;
uniform samplerCube cubeNormalMap;

in vec2 vUvs;

out vec4 frag_color;

void main() 
{	
    vec3 color = vec3( 0.0 );
    vec2 uv = vUvs * 2.0 - 1.0;
    color.xy = vUvs;

    vec3 rd = transpose( mat3x3( viewMatrix ) ) * normalize( vec3( uv.x, uv.y, -1.0 ) );

    color = texture( envMap, rd ).rgb;

    frag_color = vec4( color, 1.0 );
}
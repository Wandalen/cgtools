#version 300 es

precision mediump float;

out vec4 FragColor;

void main() 
{
	// Simple white color for silhouette detection in JFA
	FragColor = vec4( 1.0, 1.0, 1.0, 1.0 );
}

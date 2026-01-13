#version 300 es
// Set the precision for float calculations. mediump is usually sufficient for color.
precision mediump float;

in vec3 vNormal;

// Output fragment color. This will be written to the framebuffer's color attachment.
layout( location = 0 ) out vec4 frag_color;
// Output interpolated normal.
layout( location = 1 ) out vec4 normal_buffer;

void main()
{
	// Simply output a solid white color.
	// This creates a white silhouette of the object in the first rendering pass.
	// This white color is then used by the JFA initialization pass to identify
	// pixels belonging to the object.
	frag_color = vec4( 1.0, 1.0, 1.0, 1.0 );
  normal_buffer = vec4( normalize( vNormal ), 1.0 );
}

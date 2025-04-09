#version 300 es

layout (location = 0) in vec2 a_pos;
out vec2 v_tex_coord;

void main() 
{
	// Convert quad pos (-1..1) to tex coord (0..1)
	v_tex_coord = a_pos * 0.5 + 0.5; 
	gl_Position = vec4(a_pos, 0.0, 1.0);
}
#version 300 es
precision mediump float;

layout( location = 0 ) in vec4 position;
layout( location = 1 ) in vec2 uv;
out vec2 v_uv;

uniform mat4x4 mvp;

// Vertex shader only transforms quad where tiles are located as texture.
// For fragment shader need uv coords for every fragment. So uv is returned 
// here, interpolated between vertices then sended as input of fragment shader.
void main()
{
    v_uv = uv;
    gl_Position = mvp * position;
}

#version 300 es
precision mediump float;

layout( location = 0 ) in vec4 position;
layout( location = 1 ) in vec2 uv;
out vec2 v_uv;

uniform mat4x4 mvp;

void main()
{
    v_uv = uv;
    gl_Position = mvp * position;
}

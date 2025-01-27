#version 300 es 
precision mediump float;
#pragma vscode_glsllint_stage : frag;

in vec4 vColor;
out vec4 FragColor;

void main() {
    FragColor = vColor;
}
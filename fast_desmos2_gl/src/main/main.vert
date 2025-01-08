#version 430

in vec2 pos;
in vec2 inst_pos;

layout(location=0) uniform float scale;
layout(location=1) uniform float corner_scale;
layout(location=2) uniform vec2 shift;

void main() {
    gl_Position = vec4((inst_pos + pos * corner_scale + shift) * scale, 0, 1); 
}

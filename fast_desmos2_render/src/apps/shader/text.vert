#version 440

in vec2 pos;
in vec2 inst_pos;
in vec2 inst_scale;
in uint glyph_index;

flat out uint glyph_index_out;
out vec2 glyph_pos;

layout(location=0) uniform vec2 transform_scale;

layout(binding=4) buffer GlyphBounds {
	vec2 glyph_bounds[];
};

void main() {
	glyph_pos = pos;
	gl_Position = vec4((pos * inst_scale + inst_pos) * transform_scale, 0.0, 1.0);
}

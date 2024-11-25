#version 440

in vec2 pos;
flat in uint glyph_index;

out vec4 color;

layout(binding=0) buffer GlyphPoints {
	vec2 glyph_points[];
};
layout(binding=1) buffer GlyphVerbs {
	uint glyph_verbs[];
};
layout(binding=2) buffer GlyphStartses {
	uvec2 glyph_starts[];
};

void main() {
	color = vec4(1.0, 0.0, 0.0, 1.0);
}

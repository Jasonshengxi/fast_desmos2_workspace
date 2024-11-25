#version 430

layout(binding=0) readonly buffer glyphPoints {
	vec2 glyph_pos[];
};

buffer glyphVerbs {
	uint glyph_verbs[];
};

struct GlyphStarts {
	uint point;
	uint verb;
};

buffer glyphStarts {
	GlyphStarts glyph_starts[];
};

struct BoundingBox {
	vec2 minimum;
	vec2 maximum;
};

buffer glyphBounds {
	BoundingBox glyph_bounds[];
};

layout(location=0) in vec2 position;
layout(location=1) in vec2 inst_pos;
layout(location=2) in vec2 scale;

layout(location=3) in uint glyph_index;

void main() {
	BoundingBox bbox = glyph_bounds[glyph_index];
	vec2 bbox_corner = position * (bbox.maximum - bbox.minimum) + bbox.minimum;
	vec2 corner = (scale * bbox_corner) + inst_pos;
	gl_Position = vec4(corner, 0.0, 1.0);
}

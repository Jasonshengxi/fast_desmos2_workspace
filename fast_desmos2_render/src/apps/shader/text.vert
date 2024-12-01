#version 440

in vec2 pos;
in vec2 inst_pos;
in vec2 inst_scale;
in uint glyph_index_in;

flat out uint glyph_index;
out vec2 glyph_pos;

layout(location=0) uniform vec2 transform_scale;

layout(binding=3) buffer GlyphBounds {
	vec2 glyph_bounds[];
};

void main() {
	glyph_index = glyph_index_in;

	uint bound_ind = 2 * glyph_index_in;
	vec2 bound_size = glyph_bounds[bound_ind];
	vec2 bound_offset = glyph_bounds[bound_ind + 1];

	vec2 corner = bound_offset + bound_size * pos;
	glyph_pos = corner;

	vec2 new_pos = (corner * inst_scale + inst_pos) * transform_scale;
	gl_Position = vec4(new_pos, 0.0, 1.0);
}

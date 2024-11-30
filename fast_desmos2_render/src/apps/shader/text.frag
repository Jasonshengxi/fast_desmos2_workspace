#version 440

in vec2 glyph_pos;
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

layout(location=1) uniform vec4 on_color;

struct IntersectResult {
    uint intersects;
    float closest;
    uint row_intersects;
};

float infinity() {
    return 1.0 / 0.0;
}

bool is_between(float t, float lo, float hi) {
    return (t > lo) && (t < hi);
}

IntersectResult line_intersects(vec2 pos, vec2 p1, vec2 p2) {
    if (p1.y == p2.y) {
        if (pos.y == p1.y) {
            float inter_x = min(p1.x, p2.y);
            if (pos.x < inter_x) {
                return IntersectResult(1u, inter_x, 1u);
            } else {
                return IntersectResult(0u, infinity(), 0u);
            }
        } else {
            return IntersectResult(0u, infinity(), 0u);
        }
    }

    float t = (pos.y - p1.y) / (p2.y - p1.y);
    if (is_between(t, 0.0, 1.0)) {
        float x = p1.x + t * (p2.x - p1.x);
        if (x > pos.x) {
            return IntersectResult(1u, x, 1u);
        } else {
            return IntersectResult(0u, infinity(), 1u);
        }
    } else {
        return IntersectResult(0u, infinity(), 0u);
    }
}

bool quad_bounds_check(vec2 pos, vec2 p1, vec2 p2) {
    float y_min = min(p1.y, p2.y);
    float y_max = max(p1.y, p2.y);

    if (is_between(pos.y, y_min, y_max)) {
        return true;
    }
    
    float t_y_tip = p1.y / (p1.y + p2.y);

    if (!is_between(t_y_tip, 0.0, 1.0)) {
        return false;
    }

    // since the control point has been offset to 0,
    // f(t) = (1 - t)^2 * p1 + t^2 * p2
    float y_tip = (1.0 - t_y_tip) * (1.0 - t_y_tip) * p1.y + t_y_tip * t_y_tip * p2.y;

    float y_min_bound = y_min;
    float y_max_bound = y_max;
    if (y_tip > y_max_bound) {
        y_max_bound = y_tip;
    } else if (y_tip < y_min_bound) {
        y_min_bound = y_tip;
    } else {
        return false;
    }

    bool in_bounds = is_between(pos.y, y_min_bound, y_max_bound);
    return in_bounds;
}

IntersectResult quad_intersects(vec2 Pos, vec2 P1, vec2 cp, vec2 P2) {
    vec2 p1 = cp - P1;
    vec2 p2 = cp - P2;
    vec2 pos = cp - Pos;
    
    if (!quad_bounds_check(pos, p1, p2)) {
        return IntersectResult(0u, infinity(), 0u);
    }
    
    float y_sum = p1.y + p2.y;
    
    float discrim = pos.y * y_sum - p1.y * p2.y;
    if (discrim < 0.0) {
        return IntersectResult(0u, infinity(), 0u);
    }
    
    float const_term = p1.y / y_sum;
    float var_term = sqrt(discrim) / y_sum;
    
    float t1 = const_term + var_term;
    float t2 = const_term - var_term;
    
    uint result = 0;
    float intersect = infinity();
    uint row_intersect = 0;
    
    if (is_between(t1, 0.0, 1.0)) {
        float s1 = 1.0 - t1;
        float x = s1 * s1 * P1.x + 2 * t1 * s1 * cp.x + t1 * t1 * P2.x;
        if (x > Pos.x) {
            result += 1u;
            intersect = min(intersect, x);
        }
        row_intersect += 1u;
    }
    if (is_between(t2, 0.0, 1.0)) {
        float s2 = 1.0 - t2;
        float x = s2 * s2 * P1.x + 2 * t2 * s2 * cp.x + t2 * t2 * P2.x;
        
        if (x > Pos.x) {
            result += 1u;
            intersect = min(intersect, x);
        }
        row_intersect += 1u;
    }
    
    return IntersectResult(result, intersect, row_intersect);
}

void main() {
    uvec2 glyph_start = glyph_starts[glyph_index];
    uint point_start = glyph_start.x;
    uint verb_start = glyph_start.y;

    uvec2 glyph_end = glyph_starts[glyph_index + 1];
    uint point_end = glyph_end.x;
    uint verb_end = glyph_end.y;

    uint total_intersects = 0;
    uint total_row = 0;
    uint last_close = point_start;
    uint point_ind = point_start;

    vec2 start, control, end;
    IntersectResult result;
    for (uint verb_ind = verb_start; verb_ind < verb_end; verb_ind++) {
        switch (glyph_verbs[verb_ind]) {
            case 0:
                // MoveTo
                point_ind += 1;
                break;
            case 1:
                // LineTo
                start = glyph_points[point_ind - 1];
                end = glyph_points[point_ind];

                result = line_intersects(glyph_pos, start, end);
                total_intersects += result.intersects;
                total_row += result.row_intersects;

                point_ind += 1;
                break;
            case 2:
                // QuadTo
                start   = glyph_points[point_ind - 1u];
                control = glyph_points[point_ind];
                end     = glyph_points[point_ind + 1u];
                
                result = quad_intersects(glyph_pos, start, control, end);
                total_intersects += result.intersects;
                total_row += result.row_intersects;

                point_ind += 2;
                break;
            case 3:
                // Close
                start = glyph_points[point_ind - 1];
                end   = glyph_points[last_close];
                last_close = point_ind;

                result = line_intersects(glyph_pos, start, end);
                total_intersects += result.intersects;
                total_row += result.row_intersects;
                break;
            default:
                color = vec4(1.0, 0.0, 0.0, 1.0);
                return;
        }
    }

    if ((total_intersects % 2) > 0) {
        color = on_color;
    } else {
        discard;
    }
}

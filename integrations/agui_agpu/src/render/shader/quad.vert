#version 460

const mat4 INVERT_Y_AXIS_AND_SCALE = mat4(
    vec4(2.0, 0.0, 0.0, 0.0),
    vec4(0.0, -2.0, 0.0, 0.0),
    vec4(0.0, 0.0, 1.0, 0.0),
    vec4(-1.0, 1.0, 0.0, 1.0)
);

layout (set = 0, binding = 0) uniform Viewport {
    vec2 size;
} viewport;

layout(location = 0) in uint layer;
layout(location = 1) in vec4 color;
layout(location = 2) in vec2 pos;

layout(location = 0) out vec2 outPos;
layout(location = 1) out uint outLayer;
layout(location = 2) out vec4 outColor;

out gl_PerVertex {
    vec4 gl_Position;
};

void main() {
    vec2 screen_pos = pos / viewport.size;
     
    gl_Position = INVERT_Y_AXIS_AND_SCALE * vec4(screen_pos.x, screen_pos.y, 0.0, 1.0);
    
    outPos = pos;
    outLayer = layer;
    outColor = color;
}
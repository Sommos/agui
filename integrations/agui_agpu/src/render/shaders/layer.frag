#version 460

layout(binding = 4) uniform DrawType {
    uint draw_type;
};

layout(binding = 5) uniform texture2D tex;
layout(binding = 6) uniform sampler textureSampler;

layout(location = 0) in vec4 inColor;
layout(location = 1) in vec2 uv;

layout(location = 0) out vec4 outColor;

void main() {
    vec4 color = texture(sampler2D(tex, textureSampler), uv);

    if(draw_type == 0) {
        color = texture(sampler2D(tex, textureSampler), uv);
    } else if(draw_type == 1) {
        float alpha = texture(sampler2D(tex, textureSampler), uv).r;

        color = vec4(1.0, 1.0, 1.0, alpha);
    }

    if(color.a <= 0.0) {
        discard;
    }

    outColor = color * inColor;
}
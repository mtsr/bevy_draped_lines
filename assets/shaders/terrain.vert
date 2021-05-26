#version 450

layout(location = 0) in vec3 Vertex_Position;
layout(location = 1) in vec3 Vertex_Normal;

layout(set = 0, binding = 0) uniform CameraViewProj {
    mat4 ViewProj;
};

layout(set = 2, binding = 0) uniform Transform {
    mat4 Model;
};

layout(set = 2, binding = 1) uniform TerrainMaterial_scale {
    float scale;
};

layout(set = 2, binding = 2) uniform TerrainMaterial_offset {
    float offset;
};

layout(location = 0) out vec3 v_WorldPosition;
layout(location = 1) out vec3 v_WorldNormal;
layout(location = 2) out vec2 v_Uv;

void main() {
    vec4 world_position = Model * vec4(Vertex_Position, 1.0);
    v_WorldPosition = world_position.xyz;

    v_WorldNormal = mat3(Model) * Vertex_Normal;

    v_Uv = vec2(0.5, world_position.y * scale + offset);

    gl_Position = ViewProj * world_position;
}

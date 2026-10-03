import bevy_sprite_render::mesh2d::vertex_output::VertexOutput;

@group(constants::MATERIAL_BIND_GROUP) @binding(0) var bevy_texture: texture_2d<f32>;
@group(constants::MATERIAL_BIND_GROUP) @binding(1) var bevy_sampler: sampler;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4f {
    return textureSample(bevy_texture, bevy_sampler, in.uv);
}

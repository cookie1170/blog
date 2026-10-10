import bevy_sprite_render::mesh2d::{vertex_output::VertexOutput, view_bindings::globals};

@group(constants::MATERIAL_BIND_GROUP) @binding(0)
var noise_texture: texture_2d<f32>;
@group(constants::MATERIAL_BIND_GROUP) @binding(1)
var noise_sampler: sampler;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4f {
    var input = length(in.uv - vec2(0.5)) * 4;
    input -= globals.time;
    input += textureSample(noise_texture, noise_sampler, in.uv) * 0.15;

    return vec4(sin(input * 6.2831));
}

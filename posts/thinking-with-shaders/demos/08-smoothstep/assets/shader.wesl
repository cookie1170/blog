import bevy_sprite_render::mesh2d::{vertex_output::VertexOutput, view_bindings::globals};

@group(constants::MATERIAL_BIND_GROUP) @binding(0)
var noise_texture: texture_2d<f32>;
@group(constants::MATERIAL_BIND_GROUP) @binding(1)
var noise_sampler: sampler;

@group(constants::MATERIAL_BIND_GROUP) @binding(2)
var<uniform> colour: vec4f;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4f {
    let input = length(in.uv - vec2(0.5)) * 4;
    input -= globals.time;
    input += textureSample(noise_texture, noise_sampler, in.uv).r * 0.15;

    let value = smoothstep(0, 0.25, sin(input * 6.2831));
    var out = colour;
    out.a *= value;
    return out;
}

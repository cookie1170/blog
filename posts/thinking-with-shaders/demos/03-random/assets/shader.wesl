import bevy_sprite_render::mesh2d::{vertex_output::VertexOutput, view_bindings::globals};

fn pcg(seed: u32) -> f32 {
    let state = seed * 747796405u + 2891336453u;
    let word = ((state >> ((state >> 28u) + 4u)) ^ state) * 277803737u;
    return f32((word >> 22u) ^ word) / 0xffffffff;
}

fn pcg_2d(seed: vec2f) -> f32 {
    const size = 256;
    let seed_1d = u32(seed.x * size) * size + u32(seed.y * size);
    return pcg(seed_1d);
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4f {
    var input = length(in.uv - vec2(0.5)) * 4;
    input -= globals.time;
    input += pcg_2d(in.uv) * 0.15;

    return vec4(sin(input * 6.2831));
}

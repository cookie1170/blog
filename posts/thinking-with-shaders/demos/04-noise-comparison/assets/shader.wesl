import bevy_sprite_render::mesh2d::{vertex_output::VertexOutput, view_bindings::globals};

// `seed` should be between 0 and 1.
fn pcg(seed: u32) -> f32 {
    let state = seed * 747796405u + 2891336453u;
    let word = ((state >> ((state >> 28u) + 4u)) ^ state) * 277803737u;
    return f32((word >> 22u) ^ word) / 0xffffffff;
}

fn pcg_2d(seed: vec2f) -> f32 {
    const size = 64;
    let seed_1d = u32(seed.x * size) * size + u32(seed.y * size);
    return pcg(seed_1d);
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4f {
    let random = pcg_2d(in.uv);
    return vec4(random);
}

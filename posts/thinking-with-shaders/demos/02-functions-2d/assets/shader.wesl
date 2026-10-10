import bevy_sprite_render::mesh2d::{vertex_output::VertexOutput, view_bindings::globals};

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4f {
    let input = length(in.uv - vec2(0.5)) * 4 - globals.time;
    return vec4(sin(input * 6.2831));
}

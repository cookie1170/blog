import bevy_sprite_render::mesh2d::vertex_output::VertexOutput;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4f {
    return vec4(in.uv, 0, 1);
}

struct MyMaterial {
    some_uniform: vec4f,
}

@group(constants::MATERIAL_BIND_GROUP) @binding(0) var<uniform> mat: MyMaterial;

@fragment
fn fragment() -> @location(0) vec4f {
    return mat.some_uniform;
}

# shader tutorial plan

## part 1: what is a shader?

a brief introduction on _what_ a shader is, why they're used (some cool pics!)

i don't think the different types of shaders should be covered here - this is meant to be a very surface level overview

## part 2: super basic fragment shaders

### what is a material?

talk about materials and meshes in bevy
i think we start with `Mesh2d` and `MeshMaterial2d` -- no need to introduce the complexity of 3d too early

### actually writing shaders

what's a fragment shader? what does it do? and how do you write one?
explain basic wgsl syntax, maybe link to [tour of wgsl] for more,
also explain wesl and how it lets you import things from bevy (`VertexOutput`)
explain how colours work in a shader (0..1, why are they vectors?), then show `return vec4(1);`

[tour of wgsl]: https://google.github.io/tour-of-wgsl/

### uniforms

then introduce uniforms, show why they're useful and show `#[derive(AsBindGroup)]`
assignment: make a shader that takes in a colour as a uniform and returns it
assignment: make a shader that switches between two colours based on a uniform

<!-- might be worth combining parts 2 and 3, and maybe even 1 -->

## part 3: textures!

### uvs

first show uvs, the basic `return vec4(i.uv, 0, 1)` shader. maybe show the individual components first?
assignment: make a shader that takes two colours, and returns one of them in the bottom half and another in the upper half
although i think that using uvs this way might be very confusing when people encounter uv mapping and stuff.
so maybe don't recommend using them as a position, just a coordinate for where the texture is sampled from.

or maybe start from the other end: each pixel has a corresponding uv value,
which is usually used to decide where it should sample a texture from. these are defined in your mesh

### textures

then show textures, don't go into samplers too much, and how to sample textures (`textureSample`)
hmm think of assignments here. dissolve shader would probably be too complex so far because it needs masking

## part 4: 1 × 0 = 0: masking

first explain alpha more, talk about alpha modes.
then talk about masking, multiply alpha by 0 to hide the pixel
assignment: take a circle mask and apply it to your previous shader with the texture
assignment: use a circular gradient and see what happens!

then talk about [`step`], show some examples of it.
then use `step` on the previous circular gradient and see what happens
assignment: use `step`-masking with some more interesing textures, like a noise texture!
assignment: change the `step` edge using uniforms!

[`step`]: https://webgpufundamentals.org/webgpu/lessons/webgpu-wgsl-function-reference.html#func-step

## part 5: mix/lerp

talk about what [`mix`], aka lerp, does, first using just numbers, then colours.
show that you can use [`bevy_sprite_render::mesh2d::view_bindings::globals.time`] to access the current time
assignment: fade between two colours using `globals.time`, hint: you can use `sin()` to make it alternate

[`mix`]: https://webgpufundamentals.org/webgpu/lessons/webgpu-wgsl-function-reference.html#func-mix
[`bevy_sprite_render::mesh2d::view_bindings::globals.time`]: https://jannik4.github.io/wesldoc_bevy/bevy_sprite_render/latest/bevy_sprite_render/mesh2d/view_bindings/var.globals.html

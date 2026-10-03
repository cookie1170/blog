+++
title = "Writing shaders"
date = "2026-10-01"
tags = ["Tutorial", "Shaders", "Bevy", "Gamedev"]
prev = "what-is-a-shader"
+++

Now that we know what a shader is, how is it made? In this and following posts we'll be using
[Bevy](https://bevy.org/) as our engine of choice, so the tutorial assumes some prior knowledge.

<!-- ugh i really gotta make a custom syntax for <details> -->
<details>
    <summary>If you don't know Bevy, here are some resources!</summary>
    <div class="details-content">
        <p>
            <a class="a" href="https://bevy.org/learn/book/">The Bevy book</a> -- A WIP version of the official Bevy book,
            which comprehensively covers the basics (and more!) of the engine.
        </p>
        <p>
            <a class="a" href="https://taintedcoders.com/">Tainted Coders</a> -- Articles about specific Bevy features, such as the ECS or UI.
        </p>
        <p>
            <a class="a" href="https://bevy.org/examples/">Bevy examples</a> -- Examples about using the engine,
            which are useful for learning how to do specific things (i.e. render 2D shapes)
        </p>
        <p>
            <a class="a" href="https://docs.rs/bevy/">The Bevy API documentation</a> -- Documentation for specific parts/features of the engine,
            which is useful for learning about using a specific type.
        </p>
    </div>
</details>

> [!WARNING]
> This tutorial is written for Bevy 0.20, which isn't yet released!
> You can use the release candidate version instead:
>
> ```toml
> bevy = "0.20.0-rc.2"
> ```

To apply a shader, you first need something to render! To start off, we'll be using 2D meshes
because they're simpler than dealing with the third dimension right off the bat

Let's take a look at the [2D shapes] example:

```rs
// Simplified for brevity
let shape = meshes.add(Rectangle::new(128.0, 128.0));

commands.spawn((
    Mesh2d(shape),
    MeshMaterial2d(materials.add(Color::WHITE))
));
```

![A white square on a grey background](basic-mesh.png "Square")

If you've seen the example before, you might have wondered about the `MeshMaterial2d(materials.add(Color::WHITE))` line.
If you exclude it, nothing gets rendered, so it's clearly important. But _what_ is it doing?

To answer that, we'll have to talk about

# Materials

As I've eluded to before, shaders are the backbone of all rendering -- without a shader, nothing is drawn.
The _material_, which is an asset implementing either the [`Material`] (for 3d)
or the [`Material2d`] (for 2d) trait, is what provides the shader, among other things.

The material used by the [2D shapes] example is [`ColorMaterial`],
which is provided by Bevy and renders the mesh with either a solid colour or a texture.

So how do we make one ourselves?

First, let's define a struct, call it something like `MyMaterial`. Then we'll derive a few traits on it:

```rs
#[derive(AsBindGroup, Asset, Reflect, Debug, Clone)]
struct MyMaterial { }
```

The [`Asset`] derive lets the type be used as an asset, similar to how we use `Mesh` above, which the material system requires.
For more information about assets, I recommend you read the [Assets] article over at Tainted Coders

I'll explain the `AsBindGroup` derive later on, don't worry about it for now.

Next, we need to actually implement [`Material2d`]! The [`Material2d`] trait gives Bevy information
about what shaders and other properties a material has. Most of its implementations don't actually do any logic --
all the methods simply return constants. For now, let's use the default implementation for everything.

```rs
impl Material2d for MyMaterial { }
```

Finally, for Bevy to recognise our material, we need to add the [`Material2dPlugin`]
with `MyMaterial` as the generic parameter to the app.

```rs
app.add_plugins(Material2dPlugin::<MyMaterial>::default());
```

Now, let's try replacing the `ColorMaterial` with our shiny new `MyMaterial`:

```rs
fn setup(
    mut materials: ResMut<Assets<MyMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut commands: Commands
) {
    let shape = meshes.add(Rectangle::new(128.0, 128.0));

    commands.spawn((
        Mesh2d(shape),
        MeshMaterial2d(materials.add(MyMaterial { }))
    ));
}
```

![A solid fuchsia coloured square](empty-material.png "Empty material")

You might expect to see nothing being rendered -- we haven't provided a shader after all --
but what actually happens is that the square is a solid fuchsia (bright pink) colour.

That's because the `Material2d` trait provides a default implementation for `fragment_shader`,
which is a shader that simply always returns this colour.

But we don't want a bright pink square in our game! So let's write a shader ourselves.

[2D shapes]: https://bevy.org/examples/2d-rendering/2d-shapes/
[`Asset`]: https://docs.rs/bevy/latest/bevy/asset/trait.Asset.html
[Assets]: https://taintedcoders.com/bevy/assets
[`Material`]: https://docs.rs/bevy/latest/bevy/pbr/trait.Material.html
[`Material2d`]: https://docs.rs/bevy/latest/bevy/sprite_render/trait.Material2d.html
[`ColorMaterial`]: https://docs.rs/bevy/latest/bevy/prelude/struct.ColorMaterial.html
[`Material2dPlugin`]: https://docs.rs/bevy/latest/bevy/sprite_render/struct.Material2dPlugin.html

# Writing a shader

Finally we get to the "writing shaders" part of "writing shaders"!

Shaders in Bevy are written in a language called [WESL](https://wesl-lang.dev/),
which is an extension of another language, called WGSL, introducing modules and imports, which we'll use later.
For the rest of the post, I'll be using the term WESL, but most things mentioned also apply to plain WGSL.

Let's create a WESL file, like `shader.wesl`, and put it in our project's `assets/` directory.

Now, to tell Bevy to use our new fragment shader, we'll have to implement the `fragment_shader` method of `Material2d`:

```rs
impl Material2d for MyMaterial {
    fn fragment_shader() -> ShaderRef {
        // `.into()` converts the asset path into a `ShaderRef`
        "shader.wesl".into()
    }
}
```

But if we run the app, it's gonna close with an error! You might see multiple errors, but the only one that actually matters is this one:

```
Caught rendering error: Validation Error

Caused by:
  In Device::create_render_pipeline, label = 'opaque_mesh2d_pipeline'
    Error matching ShaderStages(FRAGMENT) shader requirements against the pipeline
      Unable to select an entry point: no entry point was found in the provided shader module
```

It tells us that our shader has no entry point!
The _entry point_ is a function where the execution of our shader starts, similar to the `main` function in a Rust program.
For a fragment shader, it's any function decorated with the `@fragment` attribute, so let's make one!

```wesl
// The name here doesn't matter,
// but the `@fragment` attribute is required
@fragment
fn fragment() {

}
```

As we can see, functions in wesl are declared similarly to Rust -- using the `fn` keyword. But attributes use `@` instead of `#[]`.

Now let's run the app.. and hooray! It launches!

![An empty window](empty-fragment.png "Empty fragment shader")

But our mesh is currently invisible. That's because our fragment shader doesn't tell the pixel what colour it should be!
To change that, we need to make the fragment shader return something.. But what?

A colour in a shader is represented by four numbers, each in the range from 0 to 1:
the red channel, the green channel, the blue channel, and the alpha channel (aka opacity), which form an RGBA colour.

To group multiple numbers together, WESL uses _vectors_, which, unlike `Vec` in Rust, are a fixed set of a small number of values.
They're represented by the `vec2<T>` through `vec4<T>` types, which have a generic parameter. Since each colour channel ranges from 0 to 1,
the type of each channel is `f32`, so a colour's type is `vec4<f32>`, which has a shorthand form `vec4f`.

To construct a vector, we can use the `vec4` function and pass to it 4 numbers, like `vec4(1, 2, 3, 4)`.

Now let's make our `fragment` function return a `vec4f` via `-> vec4f`:

```wesl
@fragment
fn fragment() -> vec4f {
    return vec4(1, 1, 1, 1);
}
```

Let's run the app and..

```
failed to create shader module: Validation Error

Caused by:
  In Device::create_shader_module

Shader validation error: Entry point fragment at Fragment is invalid
 = Entry point arguments and return values must all have bindings


      Entry point arguments and return values must all have bindings
```

The error message here isn't entirely clear -- what's a "binding"?

To solve it, we need to understand that any return or parameter of an entry point
must have specified _locations_ for each of its fields (which we only have one of).

This ensures that the inputs and outputs of entry points are known by Bevy's renderer. In this case,
it reads the colour from location 0, so that's what we'll use for our output via the `@location` attribute:

```wesl
@fragment
fn fragment() -> @location(0) vec4f {
    //           ++++++++++++
    return vec4(1, 1, 1, 1);
}
```

![A white square](basic-shader.png "Basic shader")

Woo! We've succesfully rendered a square! Its colour is pure white, because an RGBA value of 1 on every channel is white.

> ## Exercise
>
> Play around with it! Change the values to see how it affects the colours.
>
> Try to make the square green, blue, or yellow!

But no matter how exciting a white square is to us, it's not very exciting to the players of our game.
So let's make something more interesting!

Currently one of the biggest issues with our shader is that we can't control it from our Rust code -- it always just returns white.
If we wanted to, say, change the colour based on what position the player is at, we would have to make a different shader for every colour!

But, thankfully, there's a solution!

# Bindings

_Bindings_ are a way to control the shader from our Rust code. More precisely, they're a way to pass data from Rust to the shader.
The [`AsBindGroup`] derive that I glossed over earlier is what lets us specify bindings.

The simplest type of binding is a _uniform_, which is just a plain value passed to our shader.
We can add new uniforms by simply adding a field to our material struct and decorating it with the `#[uniform]` attribute.
The attribute expects a _binding index_ for the uniform, let's start at 0:

```rs
#[derive(AsBindGroup, …)]
struct MyMaterial {
    #[uniform(0)]
    some_uniform: LinearRgba,
}
```

The types of the fields are expected to implement [`ShaderType`],
which includes types such as `f32`, `i32`, `u32`, `LinearRgba`, all the `VecN` and `MatN` types.

Notably it does _not_ include types like `bool`, `u8`, etc. -- those will have to be packed into types like `u32`,
for which you can use [`u32::to_ne_bytes`] in Rust and [`unpack4xU8`] in WESL.

Colour fields should use the [`LinearRgba`] type, which [`Color`] can be converted into.

If you want multiple fields of your struct to be passed to the shader, you should use `#[uniform]` on all of them with the same index.
They will be combined into one uniform, which can be used in WESL with a struct:

```wesl
struct MyMaterial {
    some_uniform: vec4f,
}
```

The fields in WESL should be in the same order as in Rust.

> [!WARNING]
> If you want your game to work on WebGL, the struct's size in bytes must be a multiple of 16.
> It's common to achieve this by including padding `u32` fields under `#[cfg(target_arch = "wasm32")]`, such as:
>
> ```rs
> struct MyMaterial {
>     // This only has a size of 8 bytes!
>     #[uniform(0)]
>     some_uniform: Vec2,
>     // Pad it to 12 bytes in size
>     #[uniform(0)]
>     #[cfg(target_arch = "wasm32")]
>     _padding_12b: u32,
>     // Pad it to 16 bytes in size, we're all good!
>     #[uniform(0)]
>     #[cfg(target_arch = "wasm32")]
>     _padding_16b: u32,
> }
> ```
>
> In WESL, the padding fields should have the `@if(SIXTEEN_BYTE_ALIGNMENT)` attribute:
>
> ```wesl
> struct MyMaterial {
>     some_uniform: Vec2,
>     @if(SIXTEEN_BYTE_ALIGNMENT)
>     _padding_12b: u32,
>     @if(SIXTEEN_BYTE_ALIGNMENT)
>     _padding_16b: u32,
> }
> ```
>
> This isn't needed when using WebGPU.

Now we need to declare a top-level uniform variable in WESL to hold this struct:

```wesl
@group(constants::MATERIAL_BIND_GROUP) @binding(0) var<uniform> mat: MyMaterial;
```

The uniform has two attributes:

- `@group(constants::MATERIAL_BIND_GROUP)` -- this tells WESL that our uniform
  comes from the material's _bind group_ -- a set of multiple bindings.

- `@binding(0)` -- this tells WESL what binding index our uniform is at.

Now, our shader code can use it as usual:

```wesl
@fragment
fn fragment() -> @location(0) vec4f {
    return mat.some_uniform;
}
```

Now, if we change the value of `some_uniform` on the material asset, the colour the shader outputs will change as well!

![A square changing colour based on a colour input](uniforms.webm "Uniforms")

> ## Exercise
>
> Do some interesting stuff!
>
> Make a hit flash shader -- just like the one we looked at in the first post --
> which takes in 2 colours and selects one of them based on whether a `u32` value is 0 or not!

Sadly, though, plain colours still aren't very interesting for our players.

Currently, every pixel in our shader runs the same exact logic with the exact same data,
so no matter what we try to do, it will always return the same colour.

But, as with bindings, there's a solution to that!

[`AsBindGroup`]: https://docs.rs/bevy/latest/bevy/render/render_resource/trait.AsBindGroup.html
[`ShaderType`]: https://docs.rs/bevy/latest/bevy/render/render_resource/trait.ShaderType.html
[`LinearRgba`]: https://docs.rs/bevy/latest/bevy/color/struct.LinearRgba.html
[`Color`]: https://docs.rs/bevy/latest/bevy/color/enum.Color.html
[`unpack4xU8`]: https://gpuweb.github.io/gpuweb/wgsl/#unpack4xU8-builtin
[`u32::to_ne_bytes`]: https://doc.rust-lang.org/stable/std/primitive.u32.html#method.to_ne_bytes

# Fragment inputs

So far, our fragment shader's signature has been `fn fragment() -> vec4f`.
Since the fragment shader is pure, this means that each pixel is the same as any other.

But that doesn't have to be the case! We can change our fragment shader's signature to accept some data.
Since we're using a `Mesh2d`, we can use a parameter[^param] of type [`VertexOutput`] from [`bevy_sprite_render`].
For `Mesh3d`, you'd use the [`VertexOutput`][vertex-output-3d] from [`bevy_pbr`].
The name `VertexOutput` comes from the fact that this struct is what the vertex shader outputs, which is another type of shader I'll cover later.

[^param]:
    If you remember, earlier I've mentioned that every parameter must have a `@location`.
    This is still the case! If you look at [`VertexOutput`]'s documentation, you can see that each of its fields has `@location` (or another attribute) specified!

This is where WESL's imports come in!
We can import the type using the `import` keyword followed by the path:

```wesl
import bevy_sprite_render::mesh2d::vertex_output::VertexOutput;
```

> [!TIP]
> You can see documentation for all WESL items that bevy exports using [wesldoc]!

And then use it as our fragment shader's parameter:

```wesl
@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4f {
    // …
}
```

Now let's have a look at some of [`VertexOutput`]'s fields:

- `position` -- The position of this pixel on the screen, where `0, 0` is the top left corner and `width, height` is the bottom right corner.

- `world_position` -- The position of this pixel in the world -- this will change depending on where your mesh is positioned.

- `world_normal` -- The direction away from the mesh's surface at this point. For 2D, this is usually `0, 0, 1` for all pixels, but it matters in 3D.

Now it's getting more exciting as our pixels can be different colours! But you know what's even more exciting?

[`VertexOutput`]: https://jannik4.github.io/wesldoc_bevy/bevy_sprite_render/latest/bevy_sprite_render/mesh2d/vertex_output/struct.VertexOutput.html
[`bevy_sprite_render`]: https://jannik4.github.io/wesldoc_bevy/bevy_sprite_render/latest/bevy_sprite_render/index.html
[vertex-output-3d]: https://jannik4.github.io/wesldoc_bevy/bevy_pbr/latest/bevy_pbr/render/forward_io/struct.VertexOutput.html
[`bevy_pbr`]: https://jannik4.github.io/wesldoc_bevy/bevy_pbr/latest/bevy_pbr/index.html
[wesldoc]: https://jannik4.github.io/wesldoc_bevy/

# Textures

Besides uniforms, there's another type of binding -- a _texture_. A texture is like an image[^image] that we can use in our shader.

[^image]: But textures don't have to be 2D! They can also be 1D or 3D.

To make a texture binding, we need to use a `Handle<Image>` field in our Rust struct with a new attribute: `#[texture]` and `#[sampler]`.
Both of these attributes take a binding index:

```rs
struct MyMaterial {
    #[texture(0)]
    #[sampler(1)]
    texture: Handle<Image>,
}
```

These binding indices must be different and they can't be part of our uniform struct,
they have to be separate from the uniforms because they're a different kind of binding:

```wesl
@group(constants::MATERIAL_BIND_GROUP) @binding(0) var my_texture: texture_2d<f32>;
@group(constants::MATERIAL_BIND_GROUP) @binding(1) var my_sampler: sampler;
```

The sampler decides how the texture is sampled. If you've ever heard of nearest/point and linear/bilinear filtering -- that's what the sampler decides.
You can use textures without a sampler, but it's rare and often not advised to do that.

To sample a texture in your shader, you can use the [`textureSample`] function, which takes 3 inputs: the texture, the sampler, and the UV.

UV? What's that?

## UVs

A _UV coordinate_ is a 2D vector with its components ranging from 0 to 1.
It's used to decide where a texture is sampled (read) from.

UV `0, 0` would sample from the top left corner of the texture, UV `0.5, 0.5` would sample from the centre,
while UV `1, 1` would sample from the bottom right corner.

![0,0 in the top left corner; 1,0 in the top right corner;
0,1 in the bottom left corner; 1,1 in the bottom right corner; 0.5, 0.5 in the centre](uv.png "UV coordinates")

It's important to note that UVs are a position on a _texture_, not on a mesh.
That is to say, UV `0.5, 0.5` will always correspond to the centre of a texture,
but it might be anywhere on your mesh -- it's decided by the mesh author.

---

You can access the UV coordinates for your pixel from the [`VertexOutput`]'s `uv` field:

```wesl
fn fragment(in: VertexOutput) -> @location(0) vec4f {
    return textureSample(my_texture, my_sampler, in.uv);
}
```

Yay! Our square now has a texture!

![A Minecraft grass block texture](textures.png "Texture")

But if you try a different texture, you might run into a strange issue:

![A texture with a white background instead of no background](broken-bevy.png "Wrong background")

But the image itself had no background! Why is it white?

The reason is the [_alpha mode_][`AlphaMode2d`], which decides how the renderer uses the pixel's alpha.
The default alpha mode is [`AlphaMode2d::Opaque`], which tells the renderer to ignore the alpha completely.

But we don't want that! Instead, we can use [`AlphaMode2d::Blend`][^blend],
which makes the renderer blend the colour with what's behind it based on alpha, by overriding the [`Material2d::alpha_mode`] method

[^blend]:
    The astute reader may have noticed [`AlphaMode2d::Mask`]. If that's you, good job!
    [`AlphaMode2d::Mask`] doesn't do much by itself -- it behaves the same as `Opaque` and leaves the alpha masking to the shader,
    which can be implemented by using `discard;` on pixels with a small enough alpha. For brevity, I've chosen to use `Blend` instead.

![A Bevy bird with a correctly transparent](alpha-mode.png "Fixed alpha mode")

Hooray!

> ## Exercise
>
> Play around with it! Try swapping textures, manipulating the UVs or tinting them!
>
> Recreate the hit flash shader from the first post but using a texture this time!

> [!NOTE]
>
> If you've used Bevy in 2D before, you've likely used [`Sprite`] rather than `Mesh2d`.
>
> Fortunately, materials can also be used with [`Sprite`], [PRed by yours truly](https://github.com/bevyengine/bevy/pull/25415)!
>
> Instead of implementing the `Material2d` trait, you need to implement [`MaterialExtension2d`]
> and instead of `MeshMaterial2d`, you need to use [`SpriteMaterial`].
>
> For sprite material shaders, instead of sampling textures using `textureSample`, you can import utilities from
> [`bevy_sprite_render::sprite_mesh::functions`], such as [`sample_sprite_texture`].
>
> Make sure to call [`get_final_color`] at the end to apply tint and discard pixels based on the alpha (when the `alpha_mode` is [`Mask`][`AlphaMode2d::Mask`])!

[`AlphaMode2d`]: https://docs.rs/bevy/latest/bevy/sprite_render/enum.AlphaMode2d.html
[`AlphaMode2d::Opaque`]: https://docs.rs/bevy/latest/bevy/sprite_render/enum.AlphaMode2d.html#variant.Opaque
[`AlphaMode2d::Blend`]: https://docs.rs/bevy/latest/bevy/sprite_render/enum.AlphaMode2d.html#variant.Blend
[`AlphaMode2d::Mask`]: https://docs.rs/bevy/latest/bevy/sprite_render/enum.AlphaMode2d.html#variant.Mask
[`Material2d::alpha_mode`]: https://docs.rs/bevy/0.20.0-rc.2/bevy/sprite_render/trait.Material2d.html#method.alpha_mode
[`sample_sprite_texture`]: https://jannik4.github.io/wesldoc_bevy/bevy_sprite_render/latest/bevy_sprite_render/sprite_mesh/functions/fn.sample_sprite_texture.html
[`get_final_color`]: https://jannik4.github.io/wesldoc_bevy/bevy_sprite_render/latest/bevy_sprite_render/sprite_mesh/functions/fn.get_final_color.html
[`bevy_sprite_render::sprite_mesh::functions`]: https://jannik4.github.io/wesldoc_bevy/bevy_sprite_render/latest/bevy_sprite_render/sprite_mesh/functions/index.html
[`MaterialExtension2d`]: https://docs.rs/bevy/0.20.0-rc.2/bevy/prelude/trait.MaterialExtension2d.html
[`SpriteMaterial`]: https://docs.rs/bevy/0.20.0-rc.2/bevy/prelude/struct.SpriteMaterial.html
[`Sprite`]: https://docs.rs/bevy/0.20.0-rc.2/bevy/prelude/struct.Sprite.html

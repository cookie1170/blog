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

To apply a shader, you first need something to render! To start off, we'll be using 2D meshes
because they're simpler than dealing with the third dimension right off the bat

Let's take a look at the [2D shapes] example:

```rs
// simplified for brevity
let shape = meshes.add(Rectangle::new(128.0, 128.0));

commands.spawn((
    Mesh2d(shape),
    MeshMaterial2d(materials.add(Color::WHITE))
));
```

!["A white square on a grey background"](basic-mesh.png "Square")

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
which is an extension of another language, called WGSL, introducing modules and imports.

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

To group multiple numbers together, WGSL uses _vectors_, which, unlike `Vec` in Rust, are a fixed set of a small number of values.
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

[`embedded_asset!`]: https://docs.rs/bevy/latest/bevy/asset/macro.embedded_asset.html

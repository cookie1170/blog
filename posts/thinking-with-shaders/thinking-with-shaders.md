+++
title = "Thinking with shaders"
date = "2026-10-10"
tags = ["Tutorial", "Shaders", "Bevy", "Gamedev"]
prev = "writing-shaders"
+++

Now that we've [learned how to write a shader](/writing-shaders), how do we write some that are a little more exciting?
That's what this post is going to cover!

# Graphs and maths

Let's look at some functions![^functions] Perhaps the simplest possible function is $f(x) = 1$

[^functions]:
    Here I'm using 'function' in the maths sense: a mapping from some input to a given output.
    It doesn't have to be a function in your code!

+++
expr = "1"
x = "2"
y = "0..2"
+++

It's a horizontal line! The function's y component is always the same -- the constant 1.
This function is very reminiscent of the very first shader we wrote! Both the shader
and the function always return a constant for every possible input.

```wesl
fn fragment() -> @location(0) vec4f {
    return vec4(1, 1, 1, 1);
}
```

> [!TIP]
> The `vec` functions are actually quite smart! If you use `vec4(1)`, you'll get a vector filled with ones!
>
> And if you use `vec4(some_vec3, 1)`, you'll get a `vec4` with the first 3 components from `some_vec4`, and `1` as the fouth!

But neither the function nor the shader are very exciting! So let's do something more fun!

Let's look at another function, $f(x) = x$

+++
expr = "x"
x = "2"
size = "small"
+++

What would our "shader equivalent" to this function look like?

In the first example, our function didn't do anything with $x$, but since this one does, we need to decide what $x$ should be!

A common value to use for things like this is the UV.
Be wary that, while it may look like it for our square, UVs don't have to correspond to a position on the mesh.
For our square, the UVs are mapped to the corresponding value, but that doesn't have to be the case!

Often, this value might come from a texture instead, but, for simplicity's sake, we'll just use the UV here.

```wesl
@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4f {
    // This creates a vec4 with `in.uv.x` as every component!
    return vec4(in.uv.x);
}
```

We only use `uv.x` because, for now, we'll just use one input value.

![A square with a gradient from black to white going left-to-right](functions.webp "Identity function")

As we can see here, our square's brightness increases from left to right!

As we've learned from [the UVs chapter](/writing-shaders#uvs), the X component of the UV coordinate
on a texture increases from left to right, and, due to our square's UV mapping, also on the square.

Since the brightness of a colour ranges from 0 to 1, and so does our UV, the square becomes a gradient!

Another important function we'll look at is $sin x$.

+++
expr = "sin(x)"
x = "4"
size = "full"
+++

It smoothly oscilates between -1 and 1! Its _period of oscilation_ is $2 pi$, which means that the function returns to where it started every $2 pi$ units.
To make it easier to work with, we can multiply x by $2 pi$ so that the period of oscilation becomes 1!

+++
expr = "sin(x * 2 * pi)"
x = "4"
size = "full"
+++

Now it oscilates once per value of x (on the plot, x ranges between -4 and 4)

So let's use this on our square via the [`sin`] function!

[`sin`]: https://www.w3.org/TR/WGSL/#sin-builtin

```wesl
@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4f {
    return vec4(sin(in.uv.x * 6.2831));
}
```

![A square with one white and one black half](sine.webp "Sine")

Huh. Weird.

This happens because the value of our colour in the right half of the square becomes negative!

But, since it's just a function that works on a value, it doesn't have to only be `uv.x`! Let's try something else --
how about the time elapsed since the game launched? We can access that value by importing [`bevy_sprite_render::mesh2d::view_bindings::globals`]
and accessing the `time` field.

[`bevy_sprite_render::mesh2d::view_bindings::globals`]: https://jannik4.github.io/wesldoc_bevy/bevy_sprite_render/latest/bevy_sprite_render/mesh2d/view_bindings/var.globals.html

```wesl
import bevy_sprite_render::mesh2d::view_bindings::globals;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4f {
    return vec4(sin(globals.time));
}
```

![A square changing colour from white to black](sine-time.webm "Sine of time")

It's changing colour! But currently, it's spending a lot of time as just black due to its colour value being negative,
which gets clamped to 0. Let's fix that!

Since the output of `sin` ranges from -1..1, we can add 1 to it to make it range from 0..2. Next, if we halve the result,
the range will be 0..1, which is what we want!

```wesl
fn fragment(in: VertexOuptut) -> @location(0) vec4f {
    return vec4((sin(globals.time) + 1) / 2);
}
```

![A square changing colour from white to black](sine-time-remap.webm "Remapped sine of time")

Now it looks right! Using the global time for effects that change over time is quite a common pattern in shaders.

Let's try coming back to our first `sin` example, but vary the input, say, by adding 0.1 to it!

```wesl
fn fragment(in: VertexOutput) -> @location(0) vec4f {
    let input = in.uv.x + 0.1;
    return vec4(sin(input * 6.2831));
}
```

![A white square with an off-centre black band](sine-shifted.webp "Shifted sine")

The black band has shifted! More specifically, the "window" through which we're looking at the function has shifted.
Previously, we've been looking at the function in the range 0-1, and now it's 0.1-1.1.

What happens if we add the time value instead?

![A square with moving bands of colour on it](sine-shifted-time.webm "Sine shifted over time")

It's now moving! Because the value we add to the input changes over time, so does the "window" we're looking at it through.
We can use this to mix-and-match all sorts of values!

> ## Exercise
>
> Try using some other values and functions!
>
> You can also use the output of another function as an input or combine multiple values together, such as in our shifted sine exmple!

# 2D functions

So far, all our functions have been one dimensional. That is, they only have a single scalar input.
But that's not gonna cut it for us! So let us venture into the second dimension!

2D functions aren't actually much more complicated than 1D ones -- instead of one input, there's two!
Those inputs could be coordinates, or they can be a coordinate and a time value, etc.

One such function is the humble [`length`] -- it simply returns the length of a vector, using the pythagorean theorem.
Let's try using it with our UV!

[`length`]: https://www.w3.org/TR/WGSL/#length-builtin

![A radial gradient from the top left corner](length-top-left.webp "Length")

Since the UV originates from the top-left corner, this result shouldn't be too surprising.
We can center it by subtracting 0.5 from each component, which moves the "window"
that we're looking at the function through by 0.5 units up and to the left.

```wesl
@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4f {
    return vec4(length(in.uv - vec2(0.5)));
}
```

![A radial gradient from the centre](length.webp "Centered length")

But this function can also be composed with other functions! Let's multiply its output by, say, $4 * 2 pi$ and then run it through `sin`!

```wesl
@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4f {
    let input = length(in.uv - vec2(0.5)) * 4;
    return vec4(sin(input * 6.2831));
}
```

![A radial gradient with multiple bands from the centre](sine-length.webp "Sine of length")

We can also combine it with time, just like we did earlier with the 1-dimensional version!

![A radial gradient with multiple moving bands eminating from the centre](sine-length-time.webm "Sine of length shifted by time")

This is the basis of the ultrahand shader from Tears of the Kingdom that we looked at [in the very first post](/what-is-a-shader)!
Hopefully you're starting to see how functions can be composed together to create more and more interesting effects!

Textures can be thought of as 2D functions -- they simply take an input, the UV coordinate, and return an output, the colour.

2D functions also can be made from combining two 1D functions by adding, multiplying, averaging, etc. their outputs!

> ## Exercise
>
> Play around with it! Try combining different functions, such as [`atan2`], which returns the angle of the vector (in radians)!

[`atan2`]: https://www.w3.org/TR/WGSL/#atan2-builtin

# Noise

So far, our effects have been pretty boring, we need something to spruce them up!
One such thing is _randomness_ -- it makes them less periodic, more varied and interesting to look at,
which is why it's ubiquitous in visual effects.

In graphics programming, such randomness is typically called _noise_.

But we have a problem: shaders are _deterministic_ -- they will always produce the same output given the same input.
Which means we need some source of what _looks like_ randomness -- a _pseudorandom number generator_ (PRNG).

I'm not going to go into the details of PRNGs, so we'll use the [PCG](https://www.reedbeta.com/blog/hash-functions-for-gpu-rendering/) algorithm:

```wesl
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
```

You don't really need to know the details of this. All you need to know is that it takes a number,
shuffles it around, and produces a new, seemingly unrelated number.

Let's try to use it with our `sin` example to spice it up a little.
We can base the randomness on the UV of the point.

```wesl
@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4f {
    let input = length(in.uv - vec2(0.5)) * 4 - globals.time + pcg_2d(in.uv) * 0.15;
    return vec4(sin(input * 6.2831));
}
```

![A radial gradient with fuzzy and pixelated bands eminating from the centre](random.webm "Random offset")

Although this looks interesting in its own right, it's often not what you want --
it looks very fuzzy and pixelated, like TV static, rather than smooth and natural.
The noise we used is called _white noise_, where every pixel is completely random independently of every other pixel.

But there are also other types of noise! A common one is _perlin noise_.
Let's compare white noise (left) and perlin noise (right) by displaying each of them on a square:

![TV static-like purely random brightness values compared to smoothly changing ones](noise-comparison.webp "Noise comparison")

White noise looks like TV static -- there are no patterns, no transitions; while perlin noise
(here generated using the [`noiz`] crate) looks a lot smoother -- there are no harsh jumps in colour, every change is gradual.
This makes it look a lot more _organic_ than white noise.

So let's try using perlin noise instead of white noise for our `sin` example!

We'll use this texture for our noise, which you can right click to download[^download] in case you're folowing along.

![A perlin noise texture](noise.webp:tiny "Perlin noise")

[^download]:
    The image is WebP, which might annoy some people, but it's a great format!
    Its compression is much better than PNG and it's widely supported.
    For Bevy, you simply need to enable the `webp` cargo feature.

Instead of using our PRNG function, we'll simply sample from this texture!
Most noise textures are black-and-white, which means all of their colour channels have the same value.
So, to get a regular float from it, we'll use the red channel.

> [!NOTE]
> Aside from using `.x`, `.y`, `.z`, etc. to access the components of a vector,
> we can also use `.r`, `.g`, `.b`, and `.a`, which behave the same,
> but are conventionally used when the vector represents a colour to make the intent clearer.

![A radial gradient with wavy bands eminating from the centre](noise.webm "Noisy sine")

It's now ^w^ a ^v^ y! Perlin noise is great at adding detail and animation to shaders that looks natural, yet varied.
Using noise to change a position like that is called _domain warping_ -- a very powerful shader technique.

[`noiz`]: https://crates.io/crates/noiz

# A hint of colour

Black and white is boring! Let's add some colour, shall we?

We can use the [`mix`] function in wgsl to blend between two values based on a third value ranging from 0 to 1.
It's also often referred to as `lerp` (linear interpolation).

[`mix`]: https://www.w3.org/TR/WGSL/#mix-builtin

Let's use it to mix between two colours! Reminder: colours in shaders are represented as RGBA with all channels ranging from 0 to 1,
which is represented in Rust code by the [`LinearRgba`] type.

[`LinearRgba`]: https://docs.rs/bevy/latest/bevy/color/struct.LinearRgba.html

> [!NOTE]
> Since `sin`'s output ranges between -1 and 1, we use the [`saturate`] function, which clamps it to be between 0 and 1.

[`saturate`]: https://www.w3.org/TR/WGSL/#saturate-float-builtin

```wesl
@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4f {
    let input = length(in.uv - vec2(0.5)) * 4 - globals.time
        + textureSample(noise_texture, noise_sampler, in.uv).r * 0.15;

    let value = saturate(sin(input * 6.2831));
    let colour = mix(dark, light, value);
    return vec4(colour, 1);
}
```

![A radial gradient with green wavy bands eminating from the centre](colours.webm "Colours")

Here I've chosed to use the colours `#1c4f55` and `#226b6b`

> [!TIP]
> You can use uniforms and Bevy's new [colour input widget] to edit the colours and have them change live!

[colour input widget]: https://docs.rs/bevy/latest/bevy/feathers/controls/struct.FeathersColorInput.html

> ## Exercise
>
> Once again, play around with it! Get creative -- try combining multiple noise textures,
> using different types of noise, scrolling them over time, fading between more than 2 colours, etc.
>
> You can use a noise texture to affect the position you sample another texture from, be it noise or not.
>
> It's also common to use 3D noise textures to have a 2D noise texture that can change over time by varying its third input.
>
> The possibilities are nearly endless!

# Masking

We can also use the output of a function to decide whether or not a pixel should be visible!
This is called _masking_.

The easiest way to make a pixel not render is to set the alpha to 0! Reminder: since we're changing the alpha, make sure to use [`AlphaMode2d::Blend`]!

[`AlphaMode2d::Blend`]: https://docs.rs/bevy/latest/bevy/sprite_render/enum.AlphaMode2d.html#variant.Blend

Since most functions we work with in shaders have outputs in the range of 0 to 1,
we can simply multiply the resulting alpha by our function's output to mask a colour!

```wesl
@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4f {
    let input = length(in.uv - vec2(0.5)) * 4 - globals.time
        + textureSample(noise_texture, noise_sampler, in.uv).r * 0.15;

    let value = saturate(sin(input * 6.2831));
    var colour = vec4(light, 1);
    colour.a *= value;
    return colour;
}
```

![Waves with fuzzy edges eminating from a point with no background behind them](masking.webp "Masking")

But since a function's output ranges from 0 to 1, it might be a value other than 0 and 1!
This will cause our alpha to be very "fuzzy". This is sometimes desired, but also might not be!

To help with it, we can use the [`step`] function, which takes an edge and a value,
and returns 0 if the value is below the edge, and 1 if it's above.

[`step`]: https://www.w3.org/TR/WGSL/#step-builtin

There's also a more powerful version of `step` called [`smoothstep`], which can be used to make the edges less harsh.
Instead of simply taking an edge and a value, `smoothstep` takes two edges,
the bottom edge and the top edge, and interpolates the value if its between them using the following curve

[`smoothstep`]: https://www.w3.org/TR/WGSL/#smoothstep-builtin

+++
expr = "x * x * (3.0 - 2.0 * x)"
preset = "unit"
size = "small"
+++

Let's use `smoothstep` with a bottom edge of `0` and a top edge of `0.25` to give a subtle fading to the edges.

```wesl
@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4f {
    let input = length(in.uv - vec2(0.5)) * 4 - globals.time
        + textureSample(noise_texture, noise_sampler, in.uv).r * 0.15;

    let value = smoothstep(0, 0.25, sin(input * 6.2831));
    var colour = vec4(light, 1);
    colour.a *= value;
    return colour;
}
```

![Waves with crisp but smooth edges eminating from a point](smoothstep.webm "Smoothstep")

> ## Exercise
>
> You can use masking to do some very interesting effects!
>
> Remember the dissolve effect from Balatro we looked at in the start? That's masking!
> Try recreating it yourself!

+++
title = "What is a shader?"
date = "2026-09-29"
tags = ["Tutorial", "Shaders", "Bevy", "Gamedev"]
+++

If you've spent any time wondering about visual effects in video games, you've likely heard the term _shader_ thrown around.
But that usually creates more questions than it answers -- what _is_ a shader? And, more importantly, how do I make one?
Well, that's what I'm going to answer in this series of blog posts!

# So, what _is_ a shader?

In its simplest form, a _shader_ is simply a piece of code that runs on your GPU -- that large expensive graphics card you have in your PC --
but this usually isn't a very helpful definition, so let's dig a little deeper.

The most common type of shader you'll hear about is a _fragment shader_.
A fragment shader is a piece of code that runs for _every single pixel_
that is rendered by a mesh to decide what colour it should be
(which is why they're also often called _pixel shaders_).

On a 1920x1080 screen, there are over _2 million_ pixels! That's a lot of pixels!
Your CPU would likely struggle processing each of them 60 times a second![^cpu-struggle]
That's why we need the GPU -- the **G**raphics **P**rocessing **U**nit -- to run fragment shaders.
GPUs are designed to run code in parallel, so instead of each pixel being drawn one after another,
large batches of pixels are drawn at the same time, which lets them run _much_ faster than if it was done on the CPU.

We usually hear about shaders in the context of fancy VFX -- think minecraft shaders --
but they're actually the backbone of _all_ modern rendering, both 2D and 3D.
Shaders are used to draw meshes, textures, sprites, make sure 3D geometry is positioned correctly on the screen,
and to shade it based on lighting (hence the name *shade*r).

Even though it might seem intimidating, shader code isn't very different from CPU code.
Most of the concepts you've learnt (functions, conditionals, loops, etc.) will still be relevant for shaders.

[^cpu-struggle]:
    CPU rendering is possible (typically called software rendering),
    but it's usually a lot slower and only done for compatibility with old systems.

# What can they do?

All this theory is boring! Let's look at what you can do with shaders beyond simply drawing geometry.

To start off, let's look at one of the simplest possible shaders: A hit flash

![An enemy briefly flashing white after being hit in Hollow Knight: Silksong](hit-flash.webm "Hit flash")

When the enemy gets hit, its sprite briefly flashes white to make the attack feel more impactful -- that's a shader!
It's a pretty simple one -- it's the first useful shader we'll write! -- yet it helps with game feel dramatically.

Another simple but useful shader is a "dissolve" or "burn" effect:

![Cards "dissolving" or "burning" in Balatro](dissolve.webm "Dissolve")

Aside from all the shaking and particles, the effect is pretty simple -- simply discard some pixels and leave the rest.

Let's take a look at a more complex effect: The Ultrahand from Tears of the Kingdom[^totk]

![An object being carried by the player with green waves of "magic" on it](ultrahand.webm "Ultrahand")

[^totk]: Footage used from Nintendo's ['Play around with Ultrahand'](https://www.youtube.com/watch?v=8xh2SgpLDv4)

Here shaders are used for a subtle outline around the object, the green waves of "magic"
radiating from the point where it's held, and the connection between Link's hand and the log.

Shaders are also often used for unique art styles, such as in Dead Cells -- that game is secretly 3D with a pixelation shader;
outlines; shading effects, such as cel shading; and much more.

It's important to note how all the effects shown here aren't _just_ the shader in question.
While shaders are an important part of them, a shader alone will only get you part of the way there.
All of them include extra things to make it that little bit more juicy -- such as particles, shaking, animations, etc.

With all that out of the way, let's write some shaders ourselves!

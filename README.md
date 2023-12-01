# Pibald

This project is in an extremely early state. The only reason this is being hosted on github in this state is to let me show people the source.

Pibald is intended to be a fixed-function pipeline renderer with no texture inputs and a preference for low-poly assets. The lack of texture detail will be compensated by a domain specific language of the same name that specifies shapes for the renderer to draw on models.

## Motivation
Pibald is meant to promote the creation of fast simple assets, requiring only modeling, weighting, vertex painting, and skeletal animation to be done in an external editor. Tools yet to be developed will then be used to specify, attach, and animate additional "texture" details in the pibald language. No normals, no uv mapping, and masked 2d animations (similar in capability to Blender's grease pencil) without the need or memory sink of 2d animated textures.
## Use

In it's current state, Pibald is not usable.
I intend to use it as the renderer for a solo game dev project I've been working on for about a year and a half. But someone else might want to use it for:
- A low resource project that could use a lot of unique assets
- A project intended to enable as many end users as possible to create their own content for it
- Low overhead but expressive programmer art for a demo/prototype/proof-of-concept
## Screenshots

Currently the only ding-dong thing working in here
![itsRho](/screenshots/rho.png)

## To Do

- Get basic Pibald "textures" working
- Animate Pibald "textures"
- Develop a GUI tool for making and animating Pibald "textures"
- Implement clustered lighting
- Add shadows

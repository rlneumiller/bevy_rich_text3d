# bevy_rich_text3d

Powerful alternative to `bevy_text`.

## Overview

This crate is similar to `bevy_text` but aims to be more user friendly and powerful.

Unlike `bevy_text`, this crate renders text as a `Mesh` and an `Image` atlas. This not only works
with `StandardMaterial` but also can be empowered by user defined shaders.

We render each glyph as a separate face, meaning in a vertex shader, you can easily manipulate the
position of each glyph to create text animation.

## Rich Text

TODO

## Text Fetching

We allow custom control codes like `"Do {damage} damage."`.
When a control code `"damage"` is encountered, a parser can be used to create
a `TextFetch` that reads data from the world and write to a `FetchedTextSegment`.
`TextFetchSegment` can be used as an alternative to normal string segments.

## Tech Stack

* `cosmic_text`

Cosmic text is used for layout, this portion is shared with `bevy_text`.

* `zeno`

Used for tesselation, this is the same render engine as `bevy_text`, `cosmic_text` and `swash`.
We use this crate directly since we do not use `swash`.

* `bevy`

Bevy's asset system functions as an alternative to `swash`.

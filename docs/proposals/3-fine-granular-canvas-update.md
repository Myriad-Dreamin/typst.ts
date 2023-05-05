### Fine-granular Canvas Update

Break the canvas into small pieces and update them individually to shorten E2E latency.

To finish the task we need to implement the following function:

```rust
pub struct OpaqueCanvasState;
pub fn update_canvas(&self, state: OpaqueCanvasState,
canvas: web_sys::CanvasRenderingContext2d, frame: &typst::doc::Frame) -> OpaqueCanvasState;
```

Proposing steps for non-incremental update:

- replace rust fn `image::Image::new` by canvas image rendering api
- replace rust fn`resvg::render` by canvas svg rendering api
- replace rust fn`ttf_parser::Face::outline_glyph` by unknown api, possible fill
- replace `sk::Path/Stroke` operations by canvas api
- apply clip semantics by `canvas.clip`
- replace render_svg_glyph by `canvas.fill(path)`
- replace render_bitmap_glyph by `canvas.drawImageData(bitmap)`
- antialias, dash pattern
- design OpaqueCanvasState

With CanvasState, we can do incremental update:

- determine bounds of path2d and glyph objects
- build bound tree
- diff and cache each bound tree node

Reference:

- [C++: Determine bounds of path2d and glyph objects](https://github.com/boynextdoor-cze/CG-Final-Project.git)
- [JavaScript: Determine bounds of path2d and glyph objects](https://stackoverflow.com/questions/2587751/an-algorithm-to-find-bounding-box-of-closed-bezier-curves)

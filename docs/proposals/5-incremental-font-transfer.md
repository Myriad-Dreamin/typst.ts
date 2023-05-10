### **Compiler**+**Renderer**: Incremental Font Transfer

Use [IFT](https://www.w3.org/TR/IFT/) to reduce the font transfer latency and bandwidth.

To finish the task we need to implement the following function:

```rust
pub struct FontResolver;
impl FontResolver {
pub fn resolve(&self, id: GlyphId) -> Option<&[u8]>;
}
pub fn render_glyph(&self, res: &mut FontResolver, id: GlyphId);
```

Proposing steps of IFT for multiple document:

- [ ] Simulate the rendering process and collect accessed byte range of font data.
- [ ] Determine accessed byte range of font data for each document using above algorithm.
- [ ] Attach document to send with a font bitmap, enabling client to determine byte ranges of font data to receive.
- [ ] Send the necessary font data to the client.

The byte range is aligned to 512 bytes.

Following the [IFT](https://www.w3.org/TR/IFT/) standard is optional.

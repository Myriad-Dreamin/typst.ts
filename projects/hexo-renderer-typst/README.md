# hexo-renderer-typst

Typst renderer plugin for Hexo.

### Installation

Install Hexo dependencies:

```shell
npm install @myriaddreamin/typst.ts @myriaddreamin/typst-ts-renderer @myriaddreamin/typst-ts-node-compiler
npm install hexo-renderer-typst
```

Caution: You must algin the version of all of the above packages.

### Font assets

Currently it is not configurable. Please put your font assets in one of following directory.

- `fonts`
- `assets/fonts`
- `asset/fonts`

### Target-aware compilation

The plugin will set the `sys.inputs.x-target` to `web`. You can configure your template with the variable:

````typ
/// The default target is _pdf_.
/// The compiler will set it to _web_ when rendering a dynamic layout.
///
/// Example:
/// ```typc
/// #let is-web-target() = target.starts-with("web")
/// #let is-pdf-target() = target.starts-with("pdf")
/// ```
#let target = sys.inputs.at("x-target", default: "pdf")


#show: it => {
  if target.starts-with("web") {
    // configure stuff for web
  } else {
    // configure stuff for pdf
  }

  it
}
````

### HTML export (experimental)

The post with `.html.typ` extension will be rendered by the experimental html exporter. The plugin will try to convert the typst template to HTML.

#import "/docs/cookery/book.typ": book-page

#show: book-page.with(title: "Hexo Library")

= Hexo Library

== Installation

Install Hexo dependencies:

Note: please align the version of the dependencies to same vesrion, otherwise you may get runtime rendering error.

```bash
// ensure dependencies is installed
npm install @myriaddreamin/typst.ts @myriaddreamin/typst-ts-renderer @myriaddreamin/typst-ts-node-compiler
// ensure plugin is installed
npm install hexo-renderer-typst
```

== Usage

Add your *main* files to `source/_posts` directory. And run:

```
# serve files
hexo serve
# generate files
hexo generate
```

Currently, it could only render typst documents inside of `source/_posts` (Hexo Posts) and fix typst workspace (root directory) to the root of your blog project.

=== Font assets

Currently it is not configurable. Please put your font assets in one of following directory.

- `fonts`
- `assets/fonts`
- `asset/fonts`

=== Target-aware compilation

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

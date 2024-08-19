#import "mod.typ": *

#show: book-page.with(title: "Static, Responsive rendering")

#include "claim.typ"

Example Application: #link("https://github.com/Myriad-Dreamin/typst.ts/blob/main/packages/typst.ts/index.html")[single-file], #link("https://github.com/Myriad-Dreamin/shiroa")[shiroa] and #link("https://github.com/Myriad-Dreamin/typst.ts/tree/main/projects/hexo-renderer-typst")[hexo-renderer-typst]

== How does Resonspive Rendering Work?

You must prepare a artifact for each document, containing compressed data for different theme and screen settings. The bundle size of artifacts is optimized for typst documents.

Since preparation is offline, it is suitable for static sites and blogs.

After that, we provide a renderer to extract and render artifact in browser. Obviously, it doesn't have capabilities to compile typst code.

Besides static display effect like PDF, by rendering document to canvas, The renderer also overlays SVG for responsive effect by CSS anominations, and HTML layer for semantics embedding and user interactions. If you would like to hack, the HTML elements can be also #link("https://myriad-dreamin.github.io/shiroa/format/supports/multimedia.html")[placed] in Typst document.

== Is it Really Static?

You may doubt that if a renderer is used, it is not static. Though we provide absolutely static renderer that exports typst to browser-directed SVG, official typst also provides PNG, SVG, and PDF export. They are not responsive.

For responsive rendering, it is worth to note that some metadata can be also embedded in HTML. For example, you can embed title, description meta in head to allow page previewing
in social media. Though it is not implemented, HTML elements is considered for rendered statically to help SEO.

== Prepare Artifacts: Precompiler Part

As an example, #link("https://github.com/Myriad-Dreamin/typst.ts/blob/main/projects/hexo-renderer-typst/lib/compiler.cjs")[hexo-renderer-typst] utilizes #cross-link("/guide/all-in-one-node.typ")[All-in-one Library for Node.js] to build its functions. First, it creates a dyn layout compiler for precompiling docuemnts:

```js
this.dyn = DynLayoutCompiler.fromBoxed(NodeCompiler.create(compileArgs).intoBoxed());
```

Then, it simply invokes ```js vector()``` method to compile the document:

```js
return this.dyn.vector({ mainFilePath: path });
```

== Prepare Artifacts: Typst Scripting Part

=== `x-page-width` (stable)

Retreiving the sys arguments specified by the dynamic layout compiler:

```typ
/// It is in default A4 paper size (21cm)
#let page-width = sys.inputs.at("x-page-width", default: 21cm)
```

Templating Example:

```typ
#set page(
  width: page-width,
  height: auto, // Also, for a website, we don't need pagination.
) if is-web-target;
```

=== `x-target` (unstable)

*Note: If you find `x-target` is not overridden for export web artifacts, the SSG tool you are using may not follow this convention.*

*Note: Official typst may introduce their owned method to specify target, therefore this feature may move to the native approach in future.*

Retreiving the sys arguments specified by SSG Tools:

```typ
/// The default target is _pdf_.
/// `typst.ts` will set it to _web_ when rendering a dynamic layout.
#let target = sys.inputs.at("x-target", default: "pdf")
```

Example:

```typ
#let is-web-target() = target.starts-with("web")
#let is-pdf-target() = target.starts-with("pdf")
```

== Build Tools for SSGs (Static Site Generators)

The interface to develop a tools is explored. But there are already easy-touse tools that could be used for your SSG Framework or learned for a new one:

- #link("https://github.com/Myriad-Dreamin/shiroa")[shiroa] for self-hosted simple book (documentation) site.

- #link("https://github.com/Myriad-Dreamin/typst.ts/tree/main/projects/hexo-renderer-typst")[hexo-renderer-typst] for Hexo.

- #link("https://github.com/dark-flames/apollo-typst")[apollo-typst] for Apollo.

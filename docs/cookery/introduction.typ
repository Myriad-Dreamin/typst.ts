#import "/docs/cookery/book.typ": book-page
#import "/github-pages/docs/data-flow.typ": data-flow-graph

#show: book-page.with(title: "Introduction")

= Introduction

Typst.ts is a project dedicated to bring the power of #link("https://github.com/typst/typst")[Typst] to the world of JavaScript. In short, it composes ways to compile and render your Typst document. In the scope of server-side rendering collaborated by #text(fill: rgb("#3c9123"), "server") and #text(fill: blue, "browser"), there would be a data flow like this basically:

#figure(
  {
    set text(size: 12pt)
    data-flow-graph()
  },
  caption: [Browser-side module needed: $dagger$: compiler; $dagger.double$: renderer. ],
  numbering: none,
)

Specifically, it first typically presents a typst document in three forms:

- #link("https://myriad-dreamin.github.io/typst.ts/cookery/guide/compiler/ts-cli.html")[Form1]: Render to SVG and then embed it as a high-quality vectored image directly.

- #link("https://myriad-dreamin.github.io/typst.ts/cookery/guide/compiler/ts-cli.html")[Form2]: Preprocessed to a Vector Format artifact.

- #link("https://myriad-dreamin.github.io/typst.ts/cookery/guide/compiler/serverless.html")[Form3]: Manipulate a canvas element directly.

The #emph("Form2: Vector Format") is developed specially for typst documents, and it has several fancy features: Todo: Visualized Feature:

- Data Sharing across multiple layouts.

- Artifact Streaming.

- Incremental Rendering.

// - Incremental Font Transfer

So with *Form2*, you can continue rendeing the document in different ways:

=== Static but #underline("responsive") rendering

Example Application: #link("https://github.com/Myriad-Dreamin/typst.ts/blob/main/packages/typst.ts/index.html")[single-file], #link("https://github.com/Myriad-Dreamin/typst-book")[typst-book] and #link("https://github.com/Myriad-Dreamin/typst.ts/tree/main/packages/hexo-renderer-typst")[hexo-renderer-typst]

A compressed artifact containing data for different theme and screen settings. The bundle size of artifacts is optimized for typst documents.

=== #underline("Incremental") server-side rendering

Example Application: #link("https://github.com/Enter-tainer/typst-preview-vscode")[typst-preview]

Build a server for compilation with #link("https://myriad-dreamin.github.io/typst.ts/cookery/guide/compiler/service.html")[Compiler Service], streaming the artifact, and render it incrementally.

=== #underline("Serverless") client-side rendering

Example Application: #link("https://github.com/Myriad-Dreamin/typst.ts/blob/main/packages/typst.ts/examples/compiler.html")[single-file]

Run the entire typst directly in browser, like #link("https://typst.app")[typst.app].

// Typst.ts allows you to independently run the Typst compiler and exporter (renderer) in your browser.

// You can:

// - locally run the compilation via `typst-ts-cli` to get a precompiled document,
//   - or use `typst-ts-compiler` to build your backend programmatically.
// - build your frontend using the lightweight TypeScript library `typst.ts`.
// - send the precompiled document to your readers' browsers and render it as HTML elements.

== Application

- #link("https://myriad-dreamin.github.io/typst.ts/")[A Website built with Typst.ts]

- #link("https://github.com/Enter-tainer/typst-preview-vscode")[Instant VSCode Preview Plugin]

- #link("https://www.npmjs.com/package/hexo-renderer-typst")[Renderer Plugin for Hexo, a Blog-aware Static Site Generator]

- Renderer/Component Library for #link("https://www.npmjs.com/package/@myriaddreamin/typst.ts")[JavaScript], #link("https://www.npmjs.com/package/@myriaddreamin/typst.react")[React], and #link("https://www.npmjs.com/package/@myriaddreamin/typst.angular")[Angular]

== Further reading

+ #link("https://myriad-dreamin.github.io/typst.ts/cookery/get-started.html")[Get started with Typst.ts]

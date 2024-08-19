#import "mod.typ": *

=== Static but #underline("responsive") rendering

Example Application: #link("https://github.com/Myriad-Dreamin/typst.ts/blob/main/packages/typst.ts/index.html")[single-file], #link("https://github.com/Myriad-Dreamin/shiroa")[shiroa] and #link("https://github.com/Myriad-Dreamin/typst.ts/tree/main/projects/hexo-renderer-typst")[hexo-renderer-typst]

A compressed artifact containing data for different theme and screen settings. The bundle size of artifacts is optimized for typst documents.

#cross-link("/direction/responsive.typ")[Read more.]

=== #underline("Incremental") server-side rendering

Example Application: #link("https://github.com/Enter-tainer/typst-preview-vscode")[typst-preview]

Build a server for compilation with #link("https://myriad-dreamin.github.io/typst.ts/cookery/guide/compiler/service.html")[Compiler Service], streaming the artifact, and render it incrementally.

#cross-link("/direction/incremental.typ")[Read more.]

=== #underline("Serverless") client-side rendering

Example Application: #link("https://github.com/Myriad-Dreamin/typst.ts/blob/main/github-pages/preview.html")[single-file]

Run the entire typst directly in browser, like #link("https://typst.app")[typst.app].

#cross-link("/direction/serverless.typ")[Read more.]

# Change Log

All notable changes to the reflexo and "@myriaddreamin/\*typst\*" packages will be documented in this file.

Check [Keep a Changelog](http://keepachangelog.com/) for recommendations on how to structure this file.

## v0.6.0 - [2025-04-30]

- feat: upgrade typst to v0.13.0 in https://github.com/Myriad-Dreamin/typst.ts/pull/643
- feat: update assets to v0.13.1 in https://github.com/Myriad-Dreamin/typst.ts/pull/682

### Compiler

- feat: use world implementation from tinymist in https://github.com/Myriad-Dreamin/typst.ts/pull/636
- build: remove web-time in https://github.com/Myriad-Dreamin/typst.ts/pull/664
  - This allows to use as a typst plugin

### Renderer

- feat: render labels on content in https://github.com/Myriad-Dreamin/typst.ts/pull/638
  - `#box() <label>` will render the label attribute to the rendered elements.
- feat: support image rendering attr in https://github.com/Myriad-Dreamin/typst.ts/pull/659
  - https://typst.app/docs/reference/visualize/image/#parameters-scaling

### Packages

- fix: update bad imports in https://github.com/Myriad-Dreamin/typst.ts/pull/680
- fix: better node.js wasm import in https://github.com/Myriad-Dreamin/typst.ts/pull/700
- feat: add style.css to enhanced-typst-svg bundle by @seven-mile in https://github.com/Myriad-Dreamin/typst.ts/pull/632
- feat: expose wasm file in compiler, renderer, parser package by @c0per and @Myriad-Dreamin in https://github.com/Myriad-Dreamin/typst.ts/pull/662, https://github.com/Myriad-Dreamin/typst.ts/pull/674, https://github.com/Myriad-Dreamin/typst.ts/pull/693, and https://github.com/Myriad-Dreamin/typst.ts/pull/699

### Package: vite-plugin-typst (New)

- feat: init vite-plugin-typst in https://github.com/Myriad-Dreamin/typst.ts/pull/648
- feat: add typst-cli option by @sjfhsjfh in https://github.com/Myriad-Dreamin/typst.ts/pull/650

### Package: typst.ts

- fix: load fonts and concurrency in https://github.com/Myriad-Dreamin/typst.ts/pull/701
- test: add smoke tests in https://github.com/Myriad-Dreamin/typst.ts/pull/684
- test: default compile options in https://github.com/Myriad-Dreamin/typst.ts/pull/702
- test: break change for v0.6.0 and test vectors to build in https://github.com/Myriad-Dreamin/typst.ts/pull/704
- feat: expose `$typst` in root in https://github.com/Myriad-Dreamin/typst.ts/pull/685

### Package: typst.react

- build: update react to v0.19 by @mrappard in https://github.com/Myriad-Dreamin/typst.ts/pull/672

### Package: typst-ts-node-compiler

- fix: reset read cache for node apis in https://github.com/Myriad-Dreamin/typst.ts/pull/683
- test: confirm the creation_timestamp and pdf standard options are respected in https://github.com/Myriad-Dreamin/typst.ts/pull/696
- feat: support PdfStandard `a-3b` in https://github.com/Myriad-Dreamin/typst.ts/pull/698

### Misc

- fix: typo in README.md by @kxxt in https://github.com/Myriad-Dreamin/typst.ts/pull/633
- fix: make consistent variable name in binary input example documentation by @GaoCan702 in https://github.com/Myriad-Dreamin/typst.ts/pull/665
- docs: rewrite get started in https://github.com/Myriad-Dreamin/typst.ts/pull/671
- docs: rewrite rust service docs in https://github.com/Myriad-Dreamin/typst.ts/pull/673
- docs: rewrite all-in-one js library docs in https://github.com/Myriad-Dreamin/typst.ts/pull/675
- docs: improve wording of service.typ in https://github.com/Myriad-Dreamin/typst.ts/pull/676
- docs: update get-started and all-in-one js docs in https://github.com/Myriad-Dreamin/typst.ts/pull/678
- docs: alias index.html in https://github.com/Myriad-Dreamin/typst.ts/pull/689
- docs: some broken links in docs in https://github.com/Myriad-Dreamin/typst.ts/pull/690
- docs: lite bundle must load wasm files in https://github.com/Myriad-Dreamin/typst.ts/pull/703

**Full Changelog**: https://github.com/Myriad-Dreamin/typst.ts/compare/v0.5.4...v0.6.0

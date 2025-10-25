# Change Log

All notable changes to the reflexo and "@myriaddreamin/\*typst\*" packages will be documented in this file.

Check [Keep a Changelog](http://keepachangelog.com/) for recommendations on how to structure this file.

## v0.6.1 - [2025-07-22]

### Packages

- (Fix) Resetting before using high-level compile/renderer APIs in https://github.com/Myriad-Dreamin/typst.ts/pull/778
- (Fix) Iterating rects in labelled content in https://github.com/Myriad-Dreamin/typst.ts/pull/783
- (Fix) Correct typing of compile format in https://github.com/Myriad-Dreamin/typst.ts/pull/790
- (Change) Removing `createTypstSvgRenderer` in https://github.com/Myriad-Dreamin/typst.ts/pull/779
- (Test) Testing renderer initialization in https://github.com/Myriad-Dreamin/typst.ts/pull/791
- (Test) Adding all renderer tests in https://github.com/Myriad-Dreamin/typst.ts/pull/792
- Added `set_fonts` API in https://github.com/Myriad-Dreamin/typst.ts/pull/780
- Supported compile with root argument in https://github.com/Myriad-Dreamin/typst.ts/pull/781
- Supported query with html target in
  https://github.com/Myriad-Dreamin/typst.ts/pull/786 and https://github.com/Myriad-Dreamin/typst.ts/pull/788
- Supported load fonts on demand in https://github.com/Myriad-Dreamin/typst.ts/pull/787
- Provided snapshot API in https://github.com/Myriad-Dreamin/typst.ts/pull/777

### Compiler

- Implemented typst2hast in https://github.com/Myriad-Dreamin/typst.ts/pull/743

### rustdoc-typst-demo (New)

- Added [`rustdoc-typst-demo`](https://github.com/Myriad-Dreamin/typst.ts/tree/main/projects/rustdoc-typst-demo) in https://github.com/Myriad-Dreamin/typst.ts/pull/725

### Package: typst.ts

- (Fix) Fixed race condition in snippet lib in https://github.com/Myriad-Dreamin/typst.ts/pull/725

### Package: typst.react

- (Fix) Not using property 'local-fonts', which is missed in Firefox by @caterpillar-1 in https://github.com/Myriad-Dreamin/typst.ts/pull/724

* feat: add css format to published files in typst.react by @shipurjan in https://github.com/Myriad-Dreamin/typst.ts/pull/765

### Package: typst.vue3

- (Fix) Preventing reinitialization of compiler and renderer options during HMR by
  @bryarrow in https://github.com/Myriad-Dreamin/typst.ts/pull/773
- (Fix) Fixed incorrect Typst source code change listener by @bryarrow in https://github.com/Myriad-Dreamin/typst.ts/pull/767
- Generating ESM and type declarations for publishing in https://github.com/Myriad-Dreamin/typst.ts/pull/776

### Package: typst-ts-node-compiler

- Moving the watch lock after compilation in https://github.com/Myriad-Dreamin/typst.ts/pull/727

### Misc

- Fixed a typo in responsive.typ by @shigma in https://github.com/Myriad-Dreamin/typst.ts/pull/719
- Updated serverless render example to import correct file from jsdelivr by @mosaleh-dev in https://github.com/Myriad-Dreamin/typst.ts/pull/733
- Updated docs for all-in-one bundle in https://github.com/Myriad-Dreamin/typst.ts/pull/726

* docs: visualize asset sizes by @YDX-2147483647 in https://github.com/Myriad-Dreamin/typst.ts/pull/742

**Full Changelog**: https://github.com/Myriad-Dreamin/typst.ts/compare/v0.6.0...v0.6.1

## v0.6.0 - [2025-04-30]

The v0.5.5 was not released because typst v0.13.0 comes before the v0.5.5 release, so we decided to skip it. The next release will be v0.6.0.

- Bumped typst to v0.13.1 in https://github.com/Myriad-Dreamin/typst.ts/pull/643 and https://github.com/Myriad-Dreamin/typst.ts/pull/661
- Bumped assets to v0.13.1 in https://github.com/Myriad-Dreamin/typst.ts/pull/682

### Compiler

- Using world implementation from tinymist in https://github.com/Myriad-Dreamin/typst.ts/pull/636
- Removed `web-time` crate dependency in https://github.com/Myriad-Dreamin/typst.ts/pull/664
  - This allows to use this library in a typst plugin

### Renderer

- Rendering labels on content in https://github.com/Myriad-Dreamin/typst.ts/pull/638
  - `#box() <label>` will render the label attribute to the rendered elements.
- Supported image rendering attr in https://github.com/Myriad-Dreamin/typst.ts/pull/659
  - https://typst.app/docs/reference/visualize/image/#parameters-scaling

### Packages

- (Fix) Made better wasm loader in Node.js in https://github.com/Myriad-Dreamin/typst.ts/pull/700
  - This was giving error in Node.js 22
- Updating bad `exports` fields in `package.json` files in https://github.com/Myriad-Dreamin/typst.ts/pull/680
- Adding style.css to enhanced-typst-svg bundle by @seven-mile in https://github.com/Myriad-Dreamin/typst.ts/pull/632
- Exposing wasm file in compiler, renderer, parser package by @c0per and @Myriad-Dreamin in https://github.com/Myriad-Dreamin/typst.ts/pull/662, https://github.com/Myriad-Dreamin/typst.ts/pull/674, https://github.com/Myriad-Dreamin/typst.ts/pull/693, and https://github.com/Myriad-Dreamin/typst.ts/pull/699

### Package: vite-plugin-typst (New)

- Initialized vite-plugin-typst in https://github.com/Myriad-Dreamin/typst.ts/pull/648
- Added typst-cli option by @sjfhsjfh in https://github.com/Myriad-Dreamin/typst.ts/pull/650

### Package: typst.ts

- (Fix) Loaded fonts and concurrency in https://github.com/Myriad-Dreamin/typst.ts/pull/701
- (Break Change) default compile options in https://github.com/Myriad-Dreamin/typst.ts/pull/702
- (Break Change) break change `compile` API for v0.6.0 and test vectors to build in https://github.com/Myriad-Dreamin/typst.ts/pull/704
- (Test) Adding smoke tests in https://github.com/Myriad-Dreamin/typst.ts/pull/684
- Exposing `$typst` in root in https://github.com/Myriad-Dreamin/typst.ts/pull/685

### Package: typst.react

- Updating react peer dependency to v0.19 by @mrappard in https://github.com/Myriad-Dreamin/typst.ts/pull/672

### Package: typst-ts-node-compiler

- (Fix) Resetting read cache for node apis in https://github.com/Myriad-Dreamin/typst.ts/pull/683
- (Test) Added tests to confirm the `creation_timestamp` and `pdfStandard` options are respected in https://github.com/Myriad-Dreamin/typst.ts/pull/696
- Supported PdfStandard `a-3b` in https://github.com/Myriad-Dreamin/typst.ts/pull/698

### Misc

- (Fix) Fixed a typo in README.md by @kxxt in https://github.com/Myriad-Dreamin/typst.ts/pull/633
- (Fix) Made consistent variable name in binary input example documentation by @GaoCan702 in https://github.com/Myriad-Dreamin/typst.ts/pull/665
- Rewrote user documentations in https://github.com/Myriad-Dreamin/typst.ts/pull/671, https://github.com/Myriad-Dreamin/typst.ts/pull/673, https://github.com/Myriad-Dreamin/typst.ts/pull/675, https://github.com/Myriad-Dreamin/typst.ts/pull/676, https://github.com/Myriad-Dreamin/typst.ts/pull/678, https://github.com/Myriad-Dreamin/typst.ts/pull/689, https://github.com/Myriad-Dreamin/typst.ts/pull/690, and https://github.com/Myriad-Dreamin/typst.ts/pull/703

**Full Changelog**: https://github.com/Myriad-Dreamin/typst.ts/compare/v0.5.4...v0.6.0

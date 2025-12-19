# Change Log

All notable changes to the reflexo and "@myriaddreamin/\*typst\*" packages will be documented in this file.

Check [Keep a Changelog](http://keepachangelog.com/) for recommendations on how to structure this file.

## v0.7.0 - [2025-10-28]

- Bumped typst to v0.14.2 in
  https://github.com/Myriad-Dreamin/typst.ts/pull/796,
  https://github.com/Myriad-Dreamin/typst.ts/pull/811, and
  https://github.com/Myriad-Dreamin/typst.ts/pull/814

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
- (Fix) Respecting wrapper script passed on initialization in https://github.com/Myriad-Dreamin/typst.ts/pull/804
- Added PDF standards supported in typst v0.14 in https://github.com/Myriad-Dreamin/typst.ts/pull/800
- Added pdf tags options in https://github.com/Myriad-Dreamin/typst.ts/pull/803

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

**Full Changelog**: https://github.com/Myriad-Dreamin/typst.ts/compare/v0.6.0...v0.7.0

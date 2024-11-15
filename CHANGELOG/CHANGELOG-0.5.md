# v0.5.0

## Changelog since v0.5.0

**Full Changelog**: https://github.com/Myriad-Dreamin/typst.ts/compare/v0.4.1...v0.5.0

## New Contributors

- @sinchang made their first contribution in https://github.com/Myriad-Dreamin/typst.ts/pull/438
- @winstonewert made their first contribution in https://github.com/Myriad-Dreamin/typst.ts/pull/482
- @quank123wip made their first contribution in https://github.com/Myriad-Dreamin/typst.ts/pull/555
- @Loosetooth made their first contribution in https://github.com/Myriad-Dreamin/typst.ts/pull/560
- @oluceps made their first contribution in https://github.com/Myriad-Dreamin/typst.ts/pull/584

### Security Notes

No new security note.

- Bumped typst to 0.12.0 in https://github.com/Myriad-Dreamin/typst.ts/pull/593

### Package: @myriaddreamin/typst.node (New)

- Initialized typst.node in https://github.com/Myriad-Dreamin/typst.ts/pull/460, https://github.com/Myriad-Dreamin/typst.ts/pull/463, and https://github.com/Myriad-Dreamin/typst.ts/pull/464
- (Fix) Mutating entry state correctly by @seven-mile in https://github.com/Myriad-Dreamin/typst.ts/pull/550
- Added pdf options to typst.node in https://github.com/Myriad-Dreamin/typst.ts/pull/552
- Simplifying node api in https://github.com/Myriad-Dreamin/typst.ts/pull/558

### Package: rehype-typst (New)

- Initialized rehype-typst by @Enter-tainer in https://github.com/Myriad-Dreamin/typst.ts/pull/435, https://github.com/Myriad-Dreamin/typst.ts/pull/436, and https://github.com/Myriad-Dreamin/typst.ts/pull/437
- Using typst.node in rehype-typst in https://github.com/Myriad-Dreamin/typst.ts/pull/549
- Added links to readme by @Loosetooth in https://github.com/Myriad-Dreamin/typst.ts/pull/560

### Package: @myriaddreamin/typst.react (New)

- Supported react 18 by @sinchang in https://github.com/Myriad-Dreamin/typst.ts/pull/438
- Updated to React 18 client rendering APIs in demo by @sinchang in https://github.com/Myriad-Dreamin/typst.ts/pull/442

### Package: @myriaddreamin/typst.vue3 (New)

- Initialized typst-vue3 (simplified component) by @quank123wip in https://github.com/Myriad-Dreamin/typst.ts/pull/555

### Package: @myriaddreamin/typst.solid (New)

- Initialized typst.solid by @oluceps in https://github.com/Myriad-Dreamin/typst.ts/pull/584

### Package: hexo-renderer-typst

- Using typst.node in hexo-renderer-typst in https://github.com/Myriad-Dreamin/typst.ts/pull/471
- Added `typst query` and export customize points for scripts in https://github.com/Myriad-Dreamin/typst.ts/pull/576 and https://github.com/Myriad-Dreamin/typst.ts/pull/588
- (Fix) Deleted unused variables in processor in https://github.com/Myriad-Dreamin/typst.ts/pull/592

### CLI

- Supported reading input from stdin in https://github.com/Myriad-Dreamin/typst.ts/pull/495

### Compiler API

- Using random main file path for svg by mainContent in https://github.com/Myriad-Dreamin/typst.ts/pull/491
- Exposing experimental incremental api in https://github.com/Myriad-Dreamin/typst.ts/pull/445
- Supporting `sys.inputs` in https://github.com/Myriad-Dreamin/typst.ts/pull/595

### Renderer API

- (Fix) Unified inconsistent `pixelPerPt` across packages in https://github.com/Myriad-Dreamin/typst.ts/pull/450
- (BreakChange) Using sys.args to control layout in https://github.com/Myriad-Dreamin/typst.ts/pull/540
  - See [Prepare Artifacts: Typst Scripting Part](<https://myriad-dreamin.github.io/typst.ts/cookery/direction/responsive.html#label-x-page-width%20(stable)>) for more information.
- Replacing inline svg with html command calls in https://github.com/Myriad-Dreamin/typst.ts/pull/541
  - To improve security.

### Compiler, Rust Part

- Using span based text selection in https://github.com/Myriad-Dreamin/typst.ts/pull/447
- (Fix) Ensuring `Send + Sync` for using comemo macros in https://github.com/Myriad-Dreamin/typst.ts/pull/451
- (Fix) Resetting diff group state in https://github.com/Myriad-Dreamin/typst.ts/pull/454
- Added debug loc definitions in https://github.com/Myriad-Dreamin/typst.ts/pull/456
- Reworked vector IR and created passes in https://github.com/Myriad-Dreamin/typst.ts/pull/459
- (Fix) consider text elements which doesn't have source location in https://github.com/Myriad-Dreamin/typst.ts/pull/461
- Resolving spans in granularity of char in https://github.com/Myriad-Dreamin/typst.ts/pull/468
- Added api for mapping src to element positions in https://github.com/Myriad-Dreamin/typst.ts/pull/469
- Removed last use of unsafe spans in https://github.com/Myriad-Dreamin/typst.ts/pull/476
- Supported wasm32-unknown target in https://github.com/Myriad-Dreamin/typst.ts/pull/484
- Exporting diagnostics objects in https://github.com/Myriad-Dreamin/typst.ts/pull/492
- Supported `sys.input` in https://github.com/Myriad-Dreamin/typst.ts/pull/494
- Pulling list of packages for world in https://github.com/Myriad-Dreamin/typst.ts/pull/499
- (Fix) Detecting not found packages correctly in https://github.com/Myriad-Dreamin/typst.ts/pull/465, https://github.com/Myriad-Dreamin/typst.ts/pull/467, and https://github.com/Myriad-Dreamin/typst.ts/pull/499
- (Fix) Blocking receiving http requests on another thread in https://github.com/Myriad-Dreamin/typst.ts/pull/500
- Generalizing and improving font resolver in https://github.com/Myriad-Dreamin/typst.ts/pull/506 and https://github.com/Myriad-Dreamin/typst.ts/pull/508
- (Fix) Converting colors from different color spaces to rgb in https://github.com/Myriad-Dreamin/typst.ts/pull/501
- Added debug information on fonts in https://github.com/Myriad-Dreamin/typst.ts/pull/510
- Using naive reparsing in https://github.com/Myriad-Dreamin/typst.ts/pull/520
- Making file watching power friendly in https://github.com/Myriad-Dreamin/typst.ts/pull/522
- (Fix) Restoring from "file not found" error after restoring deleted file in https://github.com/Myriad-Dreamin/typst.ts/pull/523
- Calculating color transforms at compile time in https://github.com/Myriad-Dreamin/typst.ts/pull/528
- Allowing world snapshot in https://github.com/Myriad-Dreamin/typst.ts/pull/545
  - To help concurrent typst tasks.
- Synchronized compile actor implementation from tinymist in https://github.com/Myriad-Dreamin/typst.ts/pull/546
- Made World parameterized by both root and entry in https://github.com/Myriad-Dreamin/typst.ts/pull/509 and https://github.com/Myriad-Dreamin/typst.ts/pull/557
- Updated compiler docs in https://github.com/Myriad-Dreamin/typst.ts/pull/559 and https://github.com/Myriad-Dreamin/typst.ts/pull/564
- Added targets or layout widths argument for dynamic layout exporter in https://github.com/Myriad-Dreamin/typst.ts/pull/562
- (Fix) Accounted for edge cases when lowering stroke in https://github.com/Myriad-Dreamin/typst.ts/pull/578
- Adjusted new sink api (typst v0.12.0) in https://github.com/Myriad-Dreamin/typst.ts/pull/594

### Dom Renderer (new)

- Initialized DOM export in https://github.com/Myriad-Dreamin/typst.ts/pull/470
- Made render priority on visible pages higher in https://github.com/Myriad-Dreamin/typst.ts/pull/474
- (Fix) Calculating dom viewport width correctly by @seven-mile in https://github.com/Myriad-Dreamin/typst.ts/pull/504
- Improved text selection fallback by @seven-mile in https://github.com/Myriad-Dreamin/typst.ts/pull/480
- Replacing legacy pdf.js usages with Sema Export in https://github.com/Myriad-Dreamin/typst.ts/pull/531
- Improved the rerendering performance on multiple-page documents in https://github.com/Myriad-Dreamin/typst.ts/pull/536

### Renderer Common

- Improved performance on text selection in https://github.com/Myriad-Dreamin/typst.ts/pull/439
- Using span based text selection in https://github.com/Myriad-Dreamin/typst.ts/pull/447
- (Fix) reset render state on `reset` call in https://github.com/Myriad-Dreamin/typst.ts/pull/452
- Removed `&nbsp;` escapes in svg export by @winstonewert in https://github.com/Myriad-Dreamin/typst.ts/pull/482
- Updated all css for `&nbsp;` escaping in https://github.com/Myriad-Dreamin/typst.ts/pull/489
- (Fix) Rendering zero-sized text elements correctly in https://github.com/Myriad-Dreamin/typst.ts/pull/556
- Added span css rules to avoid users' occasional overrides in https://github.com/Myriad-Dreamin/typst.ts/pull/575
- Added guidance docs to use rendering techniques in https://github.com/Myriad-Dreamin/typst.ts/pull/579
- (Fix) Using `Abs::pt` instead of `Abs::raw` for typst v0.12.0 in https://github.com/Myriad-Dreamin/typst.ts/pull/597

### Svg Renderer

- (Fix) Reusing reference in a transformed item in https://github.com/Myriad-Dreamin/typst.ts/pull/443
- Inlined clip path definitions in https://github.com/Myriad-Dreamin/typst.ts/pull/444
- (Fix) Setting width to zero if data is not available in https://github.com/Myriad-Dreamin/typst.ts/pull/449
- Memorizing glyph hash builder in https://github.com/Myriad-Dreamin/typst.ts/pull/457

### Canvas Renderer

- (Fix) Corrected value reference in canvas rendering in https://github.com/Myriad-Dreamin/typst.ts/pull/441
- Computing bbox of canvas elements in https://github.com/Myriad-Dreamin/typst.ts/pull/532 and https://github.com/Myriad-Dreamin/typst.ts/pull/533
- Rendering canvas with damage tracking in https://github.com/Myriad-Dreamin/typst.ts/pull/534
- Using clip-based canvas rerendering in https://github.com/Myriad-Dreamin/typst.ts/pull/535

### Misc

- Added watch renderer script in https://github.com/Myriad-Dreamin/typst.ts/pull/472
- Reimplemented safe QueryRef in https://github.com/Myriad-Dreamin/typst.ts/pull/507
- Removed excessive newline in logging in https://github.com/Myriad-Dreamin/typst.ts/pull/521
- Refactored crates in https://github.com/Myriad-Dreamin/typst.ts/pull/566, https://github.com/Myriad-Dreamin/typst.ts/pull/569, https://github.com/Myriad-Dreamin/typst.ts/pull/570, https://github.com/Myriad-Dreamin/typst.ts/pull/571, https://github.com/Myriad-Dreamin/typst.ts/pull/572, and https://github.com/Myriad-Dreamin/typst.ts/pull/573
- Switched default release profile to best performance in https://github.com/Myriad-Dreamin/typst.ts/pull/581
- Using vite instead of esbuild and webpack in https://github.com/Myriad-Dreamin/typst.ts/pull/587

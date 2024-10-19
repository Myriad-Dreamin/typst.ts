# v0.5.0 (Unreleased)

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

### Package: @myriaddreamin/typst.node (New)

- feat: add typst.node in https://github.com/Myriad-Dreamin/typst.ts/pull/460
- dev(pkg::node): add more api and comments in https://github.com/Myriad-Dreamin/typst.ts/pull/463
- fix(pkg::node): remove additional fields added by napi prepublish in https://github.com/Myriad-Dreamin/typst.ts/pull/464
- fix: entry state mutation in node compiler by @seven-mile in https://github.com/Myriad-Dreamin/typst.ts/pull/550
- feat: add pdf options to typst.node in https://github.com/Myriad-Dreamin/typst.ts/pull/552
- dev: simplify node api in https://github.com/Myriad-Dreamin/typst.ts/pull/558

### Package: rehype-typst (New)

- feat: add rehype-typst by @Enter-tainer in https://github.com/Myriad-Dreamin/typst.ts/pull/435, https://github.com/Myriad-Dreamin/typst.ts/pull/436, and https://github.com/Myriad-Dreamin/typst.ts/pull/437
- feat: use typst.node in rehype-typst in https://github.com/Myriad-Dreamin/typst.ts/pull/549
- docs: add links to rehype-typst readme by @Loosetooth in https://github.com/Myriad-Dreamin/typst.ts/pull/560

### Package: @myriaddreamin/typst.react (New)

- feat(pkg::react): support react 18 by @sinchang in https://github.com/Myriad-Dreamin/typst.ts/pull/438
- chore(pkg::react): update to React 18 client rendering APIs by @sinchang in https://github.com/Myriad-Dreamin/typst.ts/pull/442

### Package: @myriaddreamin/typst.vue3 (New)

- feat: create typst-vue3 (simplified component) by @quank123wip in https://github.com/Myriad-Dreamin/typst.ts/pull/555

### Package: @myriaddreamin/typst.solid (New)

- dev(pkg::solid): init by @oluceps in https://github.com/Myriad-Dreamin/typst.ts/pull/584

### Package: hexo-renderer-typst

- feat: hexo-renderer-typst use typst.node in https://github.com/Myriad-Dreamin/typst.ts/pull/471
- fix(hexo): delete unused variables in processor in https://github.com/Myriad-Dreamin/typst.ts/pull/592

### CLI

- feat(cli): support reading input from stdin in https://github.com/Myriad-Dreamin/typst.ts/pull/495

### Compiler API

- feat: use random main file path for svg by mainContent in https://github.com/Myriad-Dreamin/typst.ts/pull/491
- feat(pkg::compiler): expose incremental api in https://github.com/Myriad-Dreamin/typst.ts/pull/445

### Renderer API

- fix(pkg::core): unify inconsistent pixel per pt in https://github.com/Myriad-Dreamin/typst.ts/pull/450
- dev: break change: use sys.args to control layout in https://github.com/Myriad-Dreamin/typst.ts/pull/540
- dev: replace inline svg with html command in https://github.com/Myriad-Dreamin/typst.ts/pull/541
- feat: add query interface and export customize points for scripts in https://github.com/Myriad-Dreamin/typst.ts/pull/576
- feat(pkg::compiler): expose incremental api in https://github.com/Myriad-Dreamin/typst.ts/pull/445

### Compiler, Rust Part

- dev(svg): use span based text selection in https://github.com/Myriad-Dreamin/typst.ts/pull/447
- fix(upstream): ensure thread-safe when using comemo macros in https://github.com/Myriad-Dreamin/typst.ts/pull/451
- dev: add debug loc definitions in https://github.com/Myriad-Dreamin/typst.ts/pull/456
- feat(core): rework vector IR and create passes in https://github.com/Myriad-Dreamin/typst.ts/pull/459
- fix(core): consider text elements which doesn't have source location in https://github.com/Myriad-Dreamin/typst.ts/pull/461
- fix(compiler): correctly detect not found packages in https://github.com/Myriad-Dreamin/typst.ts/pull/465
- revert: "fix(compiler): correctly detect not found packages" in https://github.com/Myriad-Dreamin/typst.ts/pull/467
- feat(compiler): resolve spans in granularity of char in https://github.com/Myriad-Dreamin/typst.ts/pull/468
- feat(compiler): api for mapping src to element positions in https://github.com/Myriad-Dreamin/typst.ts/pull/469
- dev: remove last use of unsafe spans in https://github.com/Myriad-Dreamin/typst.ts/pull/476
- feat(compiler): run in wasm32 unknown in https://github.com/Myriad-Dreamin/typst.ts/pull/484
- feat: export diagnostics objects in https://github.com/Myriad-Dreamin/typst.ts/pull/492
- feat(compiler): allow specifying input arguments in https://github.com/Myriad-Dreamin/typst.ts/pull/494
- feat: pull list of packages for world in https://github.com/Myriad-Dreamin/typst.ts/pull/499
- fix: blocking receiving http requests on another thread in https://github.com/Myriad-Dreamin/typst.ts/pull/500
- dev: generialze font resolver in https://github.com/Myriad-Dreamin/typst.ts/pull/506
- dev: shrink options for font resolver in https://github.com/Myriad-Dreamin/typst.ts/pull/508
- feat: let world take entry into consideration in https://github.com/Myriad-Dreamin/typst.ts/pull/509
- dev(compiler): add debug information on fonts in https://github.com/Myriad-Dreamin/typst.ts/pull/510
- dev: update benchmark and use naive reparsing in https://github.com/Myriad-Dreamin/typst.ts/pull/520
- dev: make file watching friendly in https://github.com/Myriad-Dreamin/typst.ts/pull/522
- fix: restore from "file not found" error after restoring deleted file in https://github.com/Myriad-Dreamin/typst.ts/pull/523
- feat: calculate color transforms at compile time in https://github.com/Myriad-Dreamin/typst.ts/pull/528
- feat: full support to world snapshot in https://github.com/Myriad-Dreamin/typst.ts/pull/545
- dev: improve impl of `EntryState` in https://github.com/Myriad-Dreamin/typst.ts/pull/557
- dev: update compiler docs in https://github.com/Myriad-Dreamin/typst.ts/pull/559
- docs: update get-started and revise compilers in https://github.com/Myriad-Dreamin/typst.ts/pull/564
- feat: allow setting targets or layout widths in https://github.com/Myriad-Dreamin/typst.ts/pull/562
- fix(core): reset diff group state in https://github.com/Myriad-Dreamin/typst.ts/pull/454
- fix(core): convert colors from different color spaces to rgb in https://github.com/Myriad-Dreamin/typst.ts/pull/501
- fix: edge cases for strokes in https://github.com/Myriad-Dreamin/typst.ts/pull/578

### Dom Renderer (new)

- feat(exporter::dom): init in https://github.com/Myriad-Dreamin/typst.ts/pull/470
- dev(dom): make higher render priority on visible pages in https://github.com/Myriad-Dreamin/typst.ts/pull/474
- dev: update dom export and all css for `&nbsp;` escaping in https://github.com/Myriad-Dreamin/typst.ts/pull/489
- fix: dom viewport width calculation by @seven-mile in https://github.com/Myriad-Dreamin/typst.ts/pull/504
- feat: better fallback emit by @seven-mile in https://github.com/Myriad-Dreamin/typst.ts/pull/480
- dev: replace legacy pdf js usages by sema export in https://github.com/Myriad-Dreamin/typst.ts/pull/531
- feat: improve the rerendering performance on multiple-page documents in https://github.com/Myriad-Dreamin/typst.ts/pull/536
- dev: sync compile actor implementation in https://github.com/Myriad-Dreamin/typst.ts/pull/546

### Renderer Common

- dev: improve performance on text selection in https://github.com/Myriad-Dreamin/typst.ts/pull/439
- dev(svg): use span based text selection in https://github.com/Myriad-Dreamin/typst.ts/pull/447
- Removed `&nbsp;` escapes in svg export by @winstonewert in https://github.com/Myriad-Dreamin/typst.ts/pull/482
- fix: render zero-sized text elements correctly in https://github.com/Myriad-Dreamin/typst.ts/pull/556
- feat: add span rules to avoid user overriden by simply `span` selector. in https://github.com/Myriad-Dreamin/typst.ts/pull/575
- docs: add guide to use rendering techniques in https://github.com/Myriad-Dreamin/typst.ts/pull/579

### Incremental Rendering

- fix(pkg::core): reset render state on `reset` call in https://github.com/Myriad-Dreamin/typst.ts/pull/452

### Svg Renderer

- fix(export::svg): reuse reference in a transformed item in https://github.com/Myriad-Dreamin/typst.ts/pull/443
- dev(export::svg): localize clip path definitions in https://github.com/Myriad-Dreamin/typst.ts/pull/444
- fix(exporter::svg): set width to zero if data is not available in https://github.com/Myriad-Dreamin/typst.ts/pull/449
- dev(exporter::svg): memorize glyph hash builder in https://github.com/Myriad-Dreamin/typst.ts/pull/457

### Canvas Renderer

- fix: incorrect value reference in canvas rendering in https://github.com/Myriad-Dreamin/typst.ts/pull/441

- feat: compute bbox of canvas elements in https://github.com/Myriad-Dreamin/typst.ts/pull/532
- feat: compute tight bbox of canvas path elements in https://github.com/Myriad-Dreamin/typst.ts/pull/533
- feat: render canvas with damage tracking in https://github.com/Myriad-Dreamin/typst.ts/pull/534
- feat: clip-based canvas rerendering in https://github.com/Myriad-Dreamin/typst.ts/pull/535

### Misc

- dev(exporter::svg): aggressive browser rasterization in https://github.com/Myriad-Dreamin/typst.ts/pull/448
- dev: add watch renderer script in https://github.com/Myriad-Dreamin/typst.ts/pull/472
- dev: reimplement safe QueryRef in https://github.com/Myriad-Dreamin/typst.ts/pull/507
- dev: remove excessive newline in logging in https://github.com/Myriad-Dreamin/typst.ts/pull/521
- refactor: refactor crates in https://github.com/Myriad-Dreamin/typst.ts/pull/566, https://github.com/Myriad-Dreamin/typst.ts/pull/569, https://github.com/Myriad-Dreamin/typst.ts/pull/570, https://github.com/Myriad-Dreamin/typst.ts/pull/571, https://github.com/Myriad-Dreamin/typst.ts/pull/572, and https://github.com/Myriad-Dreamin/typst.ts/pull/573
- dev: switch default release profile to best performance in https://github.com/Myriad-Dreamin/typst.ts/pull/581
- feat: use vite instead of esbuild and webpack in https://github.com/Myriad-Dreamin/typst.ts/pull/587

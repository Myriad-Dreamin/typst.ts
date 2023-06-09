# v0.2.3

Note: There will be large changes in the next version because Typst will introduce experimental package management in `v0.6.0`, so this version is the last version of the 0.2.x series.

## Changelog since v0.2.3

**Full Changelog**: https://github.com/Myriad-Dreamin/typst.ts/compare/v0.2.2...v0.2.3

### Security Notes

No new security note.

### Bug fix

- compiler: correct order of searching fonts in https://github.com/Myriad-Dreamin/typst.ts/pull/175
- compiler: reset today for each compilation in https://github.com/Myriad-Dreamin/typst.ts/pull/171

### Changes

- cli: add embedded font again in https://github.com/Myriad-Dreamin/typst.ts/pull/176
- compiler: emit plain text if out isn't a TTY in https://github.com/Myriad-Dreamin/typst.ts/pull/170
- compiler: ignore utf-8 bom in https://github.com/Myriad-Dreamin/typst.ts/pull/172
- compiler: loose relevance check for watching fs changes in https://github.com/Myriad-Dreamin/typst.ts/pull/193

### External Feature

- exporter::svg: A new SVG exporter is introduced in https://github.com/Myriad-Dreamin/typst.ts/pull/127

  - with typst semantics, responsive link in https://github.com/Myriad-Dreamin/typst.ts/pull/128, text selection in https://github.com/Myriad-Dreamin/typst.ts/pull/137.
  - with incremental rendering by Me, @seven-mile and @Enter-tainer in https://github.com/Myriad-Dreamin/typst.ts/pull/129
  - with dynamic (responsive) layout in https://github.com/Myriad-Dreamin/typst.ts/pull/141
  - with source mapping from svg elements to typst ones in https://github.com/Myriad-Dreamin/typst.ts/pull/154, https://github.com/Myriad-Dreamin/typst.ts/pull/161, and https://github.com/Myriad-Dreamin/typst.ts/pull/162

- cli: generate and watch svg with dynamic layout in https://github.com/Myriad-Dreamin/typst.ts/pull/142 and https://github.com/Myriad-Dreamin/typst.ts/pull/179
- compiler: incremental parsing in https://github.com/Myriad-Dreamin/typst.ts/pull/186

# v0.2.2

## Changelog since v0.2.2

**Full Changelog**: https://github.com/Myriad-Dreamin/typst.ts/compare/v0.2.1...v0.2.2

### Security Notes

No new security note.

### Changes

- cli: remove web_socket exporter in https://github.com/Myriad-Dreamin/typst.ts/pull/118

### External Feature

- cli: use human-panic in https://github.com/Myriad-Dreamin/typst.ts/pull/120
- pkg::core: sema layer with link annotations in https://github.com/Myriad-Dreamin/typst.ts/pull/124
- upstream: upgrade typst to v0.5.0 in https://github.com/Myriad-Dreamin/typst.ts/pull/125

# v0.2.1

## Changelog since v0.2.1

**Full Changelog**: https://github.com/Myriad-Dreamin/typst.ts/compare/v0.2.0...v0.2.1

### Bug fix

- fix(core): ir unaligned memory in https://github.com/Myriad-Dreamin/typst.ts/pull/99

### Security Notes

No new security note.

### External Feature

- compiler: eager snapshot rendering in https://github.com/Myriad-Dreamin/typst.ts/pull/63

- core: collect and export glyph info in https://github.com/Myriad-Dreamin/typst.ts/pull/112
- core: optional render text by partial fonts in https://github.com/Myriad-Dreamin/typst.ts/pull/113

- exporter::canvas error handling in https://github.com/Myriad-Dreamin/typst.ts/pull/67
- exporter::canvas is almost done (1): glyph cache and clip impl in https://github.com/Myriad-Dreamin/typst.ts/pull/65
- exporter::canvas is almost done (2): render svg and bitmap glyphs in https://github.com/Myriad-Dreamin/typst.ts/pull/86

- packages::core: customize access model in https://github.com/Myriad-Dreamin/typst.ts/pull/34
- packages::compiler: add integrated canvas renderer in https://github.com/Myriad-Dreamin/typst.ts/pull/35
- packages::renderer: font supports for cjk and emoji glyphs in https://github.com/Myriad-Dreamin/typst.ts/pull/84

- server::remote: serve remote compilation in https://github.com/Myriad-Dreamin/typst.ts/pull/54
- server::remote: load fonts in snapshot in https://github.com/Myriad-Dreamin/typst.ts/pull/60

- github-pages: init github-pages in https://github.com/Myriad-Dreamin/typst.ts/pull/37
- misc: set linguist language of typst source files in https://github.com/Myriad-Dreamin/typst.ts/pull/41

### Internal Feature

- docs: add troubleshooting docs about wasm in https://github.com/Myriad-Dreamin/typst.ts/pull/90

- test: init snapshot testing in https://github.com/Myriad-Dreamin/typst.ts/pull/72
- test: test wasm renderer with ci integration in https://github.com/Myriad-Dreamin/typst.ts/pull/76

- test: corpus for CJK font testing: add hust template in https://github.com/Myriad-Dreamin/typst.ts/pull/82
- test: corpus for math testing: add undergradmath in https://github.com/Myriad-Dreamin/typst.ts/pull/110

- test from upstream: visualize path and polygon corpus in https://github.com/Myriad-Dreamin/typst.ts/pull/79
- test from upstream: shape aspect corpora in https://github.com/Myriad-Dreamin/typst.ts/pull/85
- test from upstream: outline rendering corpora in https://github.com/Myriad-Dreamin/typst.ts/pull/89
- test from upstream: shape circle corpora in https://github.com/Myriad-Dreamin/typst.ts/pull/94
- test from upstream: layout clip corpora in https://github.com/Myriad-Dreamin/typst.ts/pull/95
- test from upstream: layout list marker corpora in https://github.com/Myriad-Dreamin/typst.ts/pull/96
- test from upstream: layout transform corpora in https://github.com/Myriad-Dreamin/typst.ts/pull/100
- test from upstream: visualize stroke corpora in https://github.com/Myriad-Dreamin/typst.ts/pull/104

# v0.2.0

## Changelog since v0.2.0

### Known Issues

- `pollster` does not work on WebAssembly, which means that we cannot run async code in a function unless it is marked as async: [Polyfill WebAssembly](https://github.com/Myriad-Dreamin/typst.ts/issues/26). This affects both development of compiler module and renderer module.

### Security Notes

No new security note.

### Changes

- `typst.ts` package's `TypstRenderer.render` method now accepts `Uint8Array` as input instead of `String`.

#### External Feature

- Program `typst-ts-cli` add commands:

  - `typst-ts-cli --VV {none,short,full,json,json-plain}`
  - `typst-ts-cli env`
  - `typst-ts-cli font measure`

- Program `typst-ts-cli compile` add flags:

  - `--watch`
  - `--trace`
  - `--web-socket`
  - `--font-path`
  - `--format {ast,ir,nothing,rmp,web_socket}`

- Program `typst-ts-cli` has been fully implemented.

- Add and implement `typst.angular` package.

- Add the ability to check for outdated artifacts using `typst_ts_core::artifact::BuildInfo`.

- (Experimental) add `typst_ts_core::artifact_ir::Artifact`, which is faster than encoding a normal artifact as JSON.

- (Experimental) add `typst_ts_core::font::FontProfile`, which can be loaded into browser compiler.

- Add `typst_ts_{ast,pdf,serde(json,rmp),ir}_exporter::Exporter`.

- Add browser compiler module and api `get_ast,get_artifact`.

- Add the ability to render individual document pages separately by browser renderer module.

- Add the ability for the browser renderer module to use system fonts from chromium `queryLocalFonts`.

- Modularize `typst.ts` package, with optional loading browser compiler module and browser renderer module.

- `typst.ts` exports compiler api `init,reset,addSource,getAst,compile`.

- `typst.ts` can now render partial pages of document.

- `typst_ts_core::artifact{,_ir}` store complete source text mapping, which improves user searching experience.

#### Internal Feature

- Unify `typst_ts_core::{Artifact,Document}Exporter` as `typst_ts_core::Exporter`.

- Add zero copy abstraction for `typst_ts_core::Exporter` (FsPathExporter, VecExporter).

- Stabilize `typst_ts_core::font::Font{Slot,Loader}`.

- Make `typst_ts_core::font::FontResolverImpl` modifiable.

- Add `typst_ts_compiler::vfs::Vfs::<AccessModel>`.

- Add `typst_ts_compiler::vfs::{{Cached,Dummy,browser::Proxy,System}AccessModel}`.

- Unify `typst_ts_compiler::{Browser,System}World` as `typst_ts_compiler::CompilerWorld`.

- Lazily add web fonts in FontData format to `typst_ts_compiler::BrowserFontSearcher`.

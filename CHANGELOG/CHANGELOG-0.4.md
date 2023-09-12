# v0.4.0-rc3

This is a major upgrade of typst.ts, so we decide to increment the minor version number. The most important change is that we have stabilized the API for TypstRenderer. We have added extensive documentation (https://www.npmjs.com/package/@myriaddreamin/typst.ts?activeTab=code), but are still working on more docs, so this release is 0.4.0-rc3 rather than 0.4.0.

We have reworked vector format (IR) in
https://github.com/Myriad-Dreamin/typst.ts/pull/317, https://github.com/Myriad-Dreamin/typst.ts/pull/324, and
https://github.com/Myriad-Dreamin/typst.ts/pull/342. As a result, there are several notable changes:

- Removed legacy artifact exporting in https://github.com/Myriad-Dreamin/typst.ts/pull/319. You can no longer get JSON output from typst.ts. Instead, use `typst.ts query` or `typst-ts-cli query` (v0.4.0+, https://github.com/Myriad-Dreamin/typst.ts/pull/286).

- Refactored Renderer API in https://github.com/Myriad-Dreamin/typst.ts/pull/336 and https://github.com/Myriad-Dreamin/typst.ts/pull/338. Existing APIs still work but will be removed in v0.5.0.

- Reworked canvas renderer with vector IR in https://github.com/Myriad-Dreamin/typst.ts/pull/318 and https://github.com/Myriad-Dreamin/typst.ts/pull/325. The new canvas renderer no longer needs to preload fonts (https://github.com/Myriad-Dreamin/typst.ts/pull/330).

## Changelog since v0.4.0-rc3

## What's Changed

**Full Changelog**: https://github.com/Myriad-Dreamin/typst.ts/compare/v0.3.1...v0.4.0-rc3

### Security Notes

No new security note.

### Bug fix

- exporter::svg: missing quote in stroke dasharray by @Enter-tainer in https://github.com/Myriad-Dreamin/typst.ts/pull/332

- core: correctly align image items in
  https://github.com/Myriad-Dreamin/typst.ts/pull/282

  - See [typst-preview: Failed to preview with figures on arm64 device](https://github.com/Enter-tainer/typst-preview/issues/77)

- core: stable sort link items when lowering in https://github.com/Myriad-Dreamin/typst.ts/pull/306

- pkg::renderer: use approx float cmp by @seven-mile in https://github.com/Myriad-Dreamin/typst.ts/pull/297

- cli: calculate abspath before linking package in https://github.com/Myriad-Dreamin/typst.ts/pull/296

- compiler: formalize font search order in https://github.com/Myriad-Dreamin/typst.ts/pull/293

- compiler: reparse prefix editing in https://github.com/Myriad-Dreamin/typst.ts/pull/316

### Changes

- build: setup typescript monorepo with turbo in https://github.com/Myriad-Dreamin/typst.ts/pull/312

  - You don't have to face the error-prone `yarn link` anymore.

- core: remove legacy artifact exporting in
  https://github.com/Myriad-Dreamin/typst.ts/pull/319

- compiler: remove deprecated resolve_with in
  https://github.com/Myriad-Dreamin/typst.ts/pull/328

- pkg::core: refactor render api in https://github.com/Myriad-Dreamin/typst.ts/pull/336 and https://github.com/Myriad-Dreamin/typst.ts/pull/338

### External Feature

- typst: sync to 0.8.0 in https://github.com/Myriad-Dreamin/typst.ts/pull/xxx

- pkg::core: adapt and export render session

- pkg::react: expose setWasmModuleInitOptions in https://github.com/Myriad-Dreamin/typst.ts/pull/311

- pkg::compiler: allow set dummy access model

- cli: add query command in https://github.com/Myriad-Dreamin/typst.ts/pull/286

- cli: add interactive query command by Me and @seven-mile in https://github.com/Myriad-Dreamin/typst.ts/pull/289

- cli: specify fonts via an environment variable `TYPST_FONT_PATHS` in https://github.com/Myriad-Dreamin/typst.ts/pull/305

- compiler: add `set_{layout_widths,extension,target}` in
  https://github.com/Myriad-Dreamin/typst.ts/pull/299,
  https://github.com/Myriad-Dreamin/typst.ts/pull/304, and in
  https://github.com/Myriad-Dreamin/typst.ts/pull/308

- compiler: embed emoji fonts for browser compiler, which will increase much bundle size

- docs: init typst.ts documentation in https://github.com/Myriad-Dreamin/typst.ts/pull/340

### Internal Feature

- core: rework vector format (IR) in
  https://github.com/Myriad-Dreamin/typst.ts/pull/317, https://github.com/Myriad-Dreamin/typst.ts/pull/324, and
  https://github.com/Myriad-Dreamin/typst.ts/pull/342

- compiler: pollyfill time support in browser

- exporter::canvas: rework with vector ir in https://github.com/Myriad-Dreamin/typst.ts/pull/318 and https://github.com/Myriad-Dreamin/typst.ts/pull/325

- corpora: auto add std test cases in https://github.com/Myriad-Dreamin/typst.ts/pull/331

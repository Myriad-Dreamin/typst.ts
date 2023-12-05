# v0.4.1

## Changelog since v0.4.1

**Full Changelog**: https://github.com/Myriad-Dreamin/typst.ts/compare/v0.4.0...v0.4.1

### Security Notes

No new security note.

### Bug fix

- compiler: compile race condition in browser in https://github.com/Myriad-Dreamin/typst.ts/pull/393

- pkg::core: add a missing await in https://github.com/Myriad-Dreamin/typst.ts/pull/394

### Changes

- cli: improve TYPST_FONT_PATHS (typst#2746) in https://github.com/Myriad-Dreamin/typst.ts/pull/432

- compiler: use fontdb to load system fonts in https://github.com/Myriad-Dreamin/typst.ts/pull/403

- compiler: compile with env on stack in https://github.com/Myriad-Dreamin/typst.ts/pull/409

- compiler: replace diag features by CompileReporter in https://github.com/Myriad-Dreamin/typst.ts/pull/413

### External Feature

- build: build: upgrade to typst v0.10.0 in https://github.com/Myriad-Dreamin/typst.ts/pull/432

- pkg::parser: init in https://github.com/Myriad-Dreamin/typst.ts/pull/401

- pkg::core: expose render canvas api in https://github.com/Myriad-Dreamin/typst.ts/pull/404 and https://github.com/Myriad-Dreamin/typst.ts/pull/405

- cli: manual generation in https://github.com/Myriad-Dreamin/typst.ts/pull/408

- cli: export pdf with timestamp in https://github.com/Myriad-Dreamin/typst.ts/pull/423

- compiler: add query, getSemanticTokens api in https://github.com/Myriad-Dreamin/typst.ts/pull/398

- compiler: add offset encoding option for getSemanticTokens in https://github.com/Myriad-Dreamin/typst.ts/pull/400

- compiler: compile with env on stack in https://github.com/Myriad-Dreamin/typst.ts/pull/409

- compiler: post process handler for dyn layout in https://github.com/Myriad-Dreamin/typst.ts/pull/428

- exporter::text: add text exporter in https://github.com/Myriad-Dreamin/typst.ts/pull/422

- exporter::svg: layout and shape text in browser in https://github.com/Myriad-Dreamin/typst.ts/pull/416 and https://github.com/Myriad-Dreamin/typst.ts/pull/420

- exporter::svg: basic left-to-right text flow detection in https://github.com/Myriad-Dreamin/typst.ts/pull/421

- exporter::svg: pull better location handler from preview in https://github.com/Myriad-Dreamin/typst.ts/pull/419

- exporter::svg: update location handler for semantic labels in https://github.com/Myriad-Dreamin/typst.ts/pull/426

### Internal Feature

- proj: add cetz-editor in https://github.com/Myriad-Dreamin/typst.ts/pull/395

- proj: init highlighter in https://github.com/Myriad-Dreamin/typst.ts/pull/402

- core: add DynGenericExporter and DynPolymorphicExporter in https://github.com/Myriad-Dreamin/typst.ts/pull/411

- core: implement ligature handling in https://github.com/Myriad-Dreamin/typst.ts/pull/414

- core: add `PageMetadata::Custom` in https://github.com/Myriad-Dreamin/typst.ts/pull/425

- core: add getCustomV1 api in https://github.com/Myriad-Dreamin/typst.ts/pull/427

- compiler: export destination path to module in https://github.com/Myriad-Dreamin/typst.ts/pull/430

- compiler: add intern support in https://github.com/Myriad-Dreamin/typst.ts/pull/429

# v0.4.0

This is a major upgrade of typst.ts, so we decide to increment the minor version number. The most important change is that we have stabilized the API for TypstRenderer. We have also added guidance to typst.ts in https://github.com/Myriad-Dreamin/typst.ts/pull/391.

One of the best features since v0.4.0 is that we provide a more user-friendly way to start exploring typst.ts, the all-in-one library apis:

```
<script type="module" src="/@myriaddreamin/typst.ts/dist/esm/contrib/all-in-one.bundle.js"></script>
<script>
  document.ready(() => {
    const svg = await $typst.svg({ mainContent: 'Hello, typst!' });
  });
</script>
```

See [All-in-one Library sample](https://github.com/Myriad-Dreamin/typst.ts/blob/main/packages/typst.ts/examples/all-in-one.html) for sample that previewing document in less than 200 LoCs and a single HTML.

We have reworked vector format (IR) in
https://github.com/Myriad-Dreamin/typst.ts/pull/317, https://github.com/Myriad-Dreamin/typst.ts/pull/324, and
https://github.com/Myriad-Dreamin/typst.ts/pull/342. As a result, there are several notable changes:

- Removed legacy artifact exporting in https://github.com/Myriad-Dreamin/typst.ts/pull/319. You can no longer get JSON output from typst.ts. Instead, use `typst.ts query` or `typst-ts-cli query` (v0.4.0+, https://github.com/Myriad-Dreamin/typst.ts/pull/286).

- Refactored Renderer API in https://github.com/Myriad-Dreamin/typst.ts/pull/336 and https://github.com/Myriad-Dreamin/typst.ts/pull/338. Existing APIs still work but will be removed in v0.5.0.

- Reworked canvas renderer with vector IR in https://github.com/Myriad-Dreamin/typst.ts/pull/318 and https://github.com/Myriad-Dreamin/typst.ts/pull/325. The new canvas renderer no longer needs to preload fonts (https://github.com/Myriad-Dreamin/typst.ts/pull/330).

## Changelog since v0.4.0

## What's Changed

**Full Changelog**: https://github.com/Myriad-Dreamin/typst.ts/compare/v0.3.1...v0.4.0

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

---

Since v0.4.0-rc3

- core: gc order in https://github.com/Myriad-Dreamin/typst.ts/pull/352

- core: hold span to/from u64 safety for users in https://github.com/Myriad-Dreamin/typst.ts/pull/361

- core: error is not send in https://github.com/Myriad-Dreamin/typst.ts/commit/05060cfe5a3bf9f0ba7404f069320cfcb3bb2aaa

- compiler: eagle check syntax of the main file in https://github.com/Myriad-Dreamin/typst.ts/pull/374

- compiler: vfs panic when file not found by @Enter-tainer in https://github.com/Myriad-Dreamin/typst.ts/pull/380

- exporter::svg: broken clip on adjacent paths in https://github.com/Myriad-Dreamin/typst.ts/pull/386

- exporter::svg: partially disable incr rendering in https://github.com/Myriad-Dreamin/typst.ts/pull/387Dreamin/typst.ts/commit/ad69d915d14f587d8e9a40300bc85f6dac4364a1

- pkg::compiler: set default dummy access model in https://github.com/Myriad-Dreamin/typst.ts/pull/364

### Changes

- build: setup typescript monorepo with turbo in https://github.com/Myriad-Dreamin/typst.ts/pull/312

  - You don't have to face the error-prone `yarn link` anymore.

- core: remove legacy artifact exporting in
  https://github.com/Myriad-Dreamin/typst.ts/pull/319

- compiler: remove deprecated resolve_with in
  https://github.com/Myriad-Dreamin/typst.ts/pull/328

- pkg::core: refactor render api in https://github.com/Myriad-Dreamin/typst.ts/pull/336 and https://github.com/Myriad-Dreamin/typst.ts/pull/338

---

Since v0.4.0-rc3

- CSS change since typst v0.9.0 in https://github.com/Myriad-Dreamin/typst.ts/pull/384

  Reference change: https://github.com/Myriad-Dreamin/typst.ts/commit/c9f185a6e16a901ae253bed2aef3c9ab1f49fd83#diff-8913391598d5e624d7d91114b7d3deb33a841833b79d7f6045258a5144abccfbL33-R36

  ```css
  - .outline_glyph path {
  + .outline_glyph path, path.outline_glyph {
    fill: var(--glyph_fill);
  }
  - .outline_glyph path {
  + .outline_glyph path, path.outline_glyph {
    transition: 0.2s fill;
  }
  ```

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

---

Since v0.4.0-rc3

- more convenient way to integrate projects in https://github.com/Myriad-Dreamin/typst.ts/pull/388

- exporter::svg: embed transparent html elements in https://github.com/Myriad-Dreamin/typst.ts/pull/379

- pkg::core: all-in-one library support in https://github.com/Myriad-Dreamin/typst.ts/commit/17d86f8a9325c62eddf59d5b52a117f1da2d3167

- pkg::core: let typst.ts work with node.js (nodenext) in https://github.com/Myriad-Dreamin/typst.ts/pull/366

- pkg::core: add option of assetUrlPrefix in https://github.com/Myriad-

- pkg::compiler: load font asset from remote in https://github.com/Myriad-Dreamin/typst.ts/pull/368

- pkg::compiler: export to pdf api in https://github.com/Myriad-Dreamin/typst.ts/pull/372

- pkg::compiler: fetch package support in https://github.com/Myriad-Dreamin/typst.ts/pull/373

- compiler: new font distribute strategy in https://github.com/Myriad-Dreamin/typst.ts/pull/362

  You can install `typst-ts-cli` by cargo since this PR:

  ```
  cargo install --locked --git https://github.com/Myriad-Dreamin/typst.ts typst-ts-cli
  ```

- compiler: add actor for watch compiler in https://github.com/Myriad-Dreamin/typst.ts/pull/371

### Internal Feature

- core: rework vector format (IR) in
  https://github.com/Myriad-Dreamin/typst.ts/pull/317, https://github.com/Myriad-Dreamin/typst.ts/pull/324, and
  https://github.com/Myriad-Dreamin/typst.ts/pull/342

- compiler: pollyfill time support in browser

- exporter::canvas: rework with vector ir in https://github.com/Myriad-Dreamin/typst.ts/pull/318 and https://github.com/Myriad-Dreamin/typst.ts/pull/325

- corpora: auto add std test cases in https://github.com/Myriad-Dreamin/typst.ts/pull/331

---

Since v0.4.0-rc3

- test: add incremental compilation fuzzer in https://github.com/Myriad-Dreamin/typst.ts/pull/370

# v0.3.1

## Changelog since v0.3.1

**Full Changelog**: https://github.com/Myriad-Dreamin/typst.ts/compare/v0.3.0...v0.3.1

### Known Issues

- exporter::svg: tyring to fix baseline of bitmap glyph, but untested in https://github.com/Myriad-Dreamin/typst.ts/pull/249

### Security Notes

- compiler: safer path_for_id in https://github.com/Myriad-Dreamin/typst.ts/pull/271

  Before this commit, a malicious package can access any file in the system.

### Bug fix

- compiler: reparse with empty content in https://github.com/Myriad-Dreamin/typst.ts/pull/236
- exporter::svg: memorize image_item rendering in https://github.com/Myriad-Dreamin/typst.ts/pull/247
- exporter::svg: correct group class name in https://github.com/Myriad-Dreamin/typst.ts/pull/251
- exporter::svg: remove unused prev correctly in https://github.com/Myriad-Dreamin/typst.ts/pull/264
- exporter::svg: correct reuse target in https://github.com/Myriad-Dreamin/typst.ts/pull/268
- compiler: ignore inotify events from output in https://github.com/Myriad-Dreamin/typst.ts/pull/261
- cli: correct logger setting in https://github.com/Myriad-Dreamin/typst.ts/pull/262
- pkg::core: link script in https://github.com/Myriad-Dreamin/typst.ts/pull/275
- compiler: only download packages in @preview in https://github.com/Myriad-Dreamin/typst.ts/pull/276
- compiler: datetime offset in https://github.com/Myriad-Dreamin/typst.ts/pull/278

### Changes

- cli: distinguish svg and svg-html export in https://github.com/Myriad-Dreamin/typst.ts/pull/259

### External Feature

- exporter::svg: experimental svg minifier in https://github.com/Myriad-Dreamin/typst.ts/pull/252
- exporter::svg: page-level partial rendering in https://github.com/Myriad-Dreamin/typst.ts/pull/263
- pkg::renderer: add renderer_build_info in https://github.com/Myriad-Dreamin/typst.ts/pull/273
- build(typst): update to v0.7.0 in https://github.com/Myriad-Dreamin/typst.ts/pull/277

### Internal Feature

- core: calculate bbox for vector items in https://github.com/Myriad-Dreamin/typst.ts/pull/239
- compiler: expose shadow files apis in https://github.com/Myriad-Dreamin/typst.ts/pull/253

# v0.3.0

## Changelog since v0.3.0

**Full Changelog**: https://github.com/Myriad-Dreamin/typst.ts/compare/v0.2.3...v0.3.0

### Security Notes

No new security note.

### Bug fix

- exporter::svg: animation only transist on fill changes in https://github.com/Myriad-Dreamin/typst.ts/pull/206

### Changes

- exporter::svg: attach defs tag by class attribute instead of id attribute in https://github.com/Myriad-Dreamin/typst.ts/pull/227

### External Feature

- typst: sync to 0.6.0 in https://github.com/Myriad-Dreamin/typst.ts/pull/198

- target: support riscv64-linux in https://github.com/Myriad-Dreamin/typst.ts/pull/207 and https://github.com/Myriad-Dreamin/typst.ts/pull/223

- cli: list packages in https://github.com/Myriad-Dreamin/typst.ts/pull/202
- cli: link/unlink packages in https://github.com/Myriad-Dreamin/typst.ts/pull/203
- cli: generate documentation site for packages packages in https://github.com/Myriad-Dreamin/typst.ts/pull/204

- hexo-renderer-typst uses Dynamic SVG exporter by Me and @seven-mile in https://github.com/Myriad-Dreamin/typst.ts/pull/197

### Internal Feature

- compiler: add `package::Registry` model in https://github.com/Myriad-Dreamin/typst.ts/pull/199

- compiler: add overlay access model in https://github.com/Myriad-Dreamin/typst.ts/pull/218

- test: init heap profiling in https://github.com/Myriad-Dreamin/typst.ts/pull/221

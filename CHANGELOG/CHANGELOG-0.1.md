# v0.1.5

## Changelog since v0.1.5

### Known Issues

No new known issue.

### Security Notes

No new security note.

### Changes

- rename `typst_renderer_ts` to `typst_ts_renderer`

#### Feature

- add `typst_ts_core::ArtifactExporter`

- canvas exporter supports rendering outline glyph, shape, and images.

- canvas exporter does not relies on rust graphics library, which eliminates the bundle size of `typst_ts_renderer` about 600KB.

#### Bugfix

- fix several unhandled cases in `typst_ts_renderer::ArtifactJsBuilder`

# v0.1.4

## Changelog since v0.1.4

### Known Issues

No new known issue.

### Security Notes

No new security note.

### Changes

#### Feature

- add docker image `myriaddreamin/typst.ts` to [ghcr.io](https://github.com/Myriad-Dreamin/typst.ts/pkgs/container/typst.ts).

- set visibility during rendering

- compiler add a format `rmp` ([rust message pack](https://docs.rs/rmp/latest/rmp/))

- introduce [canvas exporter](https://github.com/Myriad-Dreamin/typst.ts/blob/main/exporter/canvas/Cargo.toml)

#### Bugfix

# v0.1.3

## Changelog since v0.1.3

### Known Issues

No new known issue.

### Security Notes

No new security note.

### Changes

#### Feature

- introduce `@myriaddreamin/typst.angular` to provide angular component.

- reduce the size of `@myriaddreamin/typst.ts` by removing serde and serde_json from dependencies.

  - using `JSON.parse` instead of serde_json.

- attach build info (compiler name and compiler version) to artifact.

#### Bugfix

# v0.1.2

## Changelog since v0.1.2

### Known Issues

- To export typst document as pdf format, enture `typst::eval::LANG_ITEMS` must be included in `typst@0.2.0`. This can cause the overall package size to increase by approximately 6MB.

### Security Notes

No new security note.

### Changes

- The upstream `typst@0.2.0` has been merged, which causes breaking changes of artifact format.

#### Feature

- upgrade es target of `@myriaddreamin/typst.ts` to es2020.

- publish `@myriaddreamin/typst.react` to npm.

- merge [hasher change](https://github.com/typst/typst/commit/d0afba959d18d1c2c646b99e6ddd864b1a91deb2) from upstream, which seems to significantly improves performance.

- add method `runWithSession` to `@myriaddreamin/typst.ts/TypstRenderer`.

- `@myriaddreamin/typst.ts/TypstRenderer` can now render document with multiple pages.

#### Bugfix

- fix bug that document cannot scroll.

# v0.1.1

## Changelog since v0.1.1

### Known Issues

No new known issue.

### Security Notes

No new security note.

### Changes

#### Feature

- add continuous integration with github actions.

#### Bugfix

- ensure that `typst-ts-fontctl` creates the directory for copying fonts on all platform.

- fix `typst-ts-cli` not working on linux.

# v0.1.0

## Changelog since v0.1.0

### Known Issues

No new known issue.

### Security Notes

No new security note.

### Changes

#### Feature

- add program `typst-ts-cli`, with [commands: compile, font:list](https://github.com/Myriad-Dreamin/typst.ts/blob/2478df888282af09dc814a481348745c4311f98f/cli/src/lib.rs).

- add program `typst-ts-fontctl` to download font assets from typst repo, [ref](https://github.com/Myriad-Dreamin/typst.ts/blob/2478df888282af09dc814a481348745c4311f98f/contrib/fontctl/src/main.rs).

- add `typst_ts_core::Artifact` to represent a precompiled document.

- add `typst_ts_core::Artifact::to_document` to convert an artifact to a `typst::doc::Document`.

- introduce `typst_ts_core::config::WorkspaceConfig` for configure workspace for compiler.

- introduce `typst_ts_core::config::CompileOpts` for control low-level behavior of compiler.

- add `@myriaddreamin/typst.ts/createTypstRenderer(pdfjsModule): TypstRenderer`.

- add method `init` and method `render` to `@myriaddreamin/typst.ts/TypstRenderer`.

- add `@myriaddreamin/typst.ts/preloadRemoteFonts: BeforeBuildFn`.

- add `@myriaddreamin/typst.ts/preloadSystemFonts: BeforeBuildFn`.

- add `@myriaddreamin/typst.react/<TypstDocument fill?='' artifact=''>`.

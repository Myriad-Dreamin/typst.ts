# Change Log

All notable changes to the reflexo and "@myriaddreamin/*typst*" packages will be documented in this file.

Check [Keep a Changelog](http://keepachangelog.com/) for recommendations on how to structure this file.

## v0.8.0 - [2026-06-16]

- Updated the Typst integration to Typst `v0.15.0-rc.1` in https://github.com/Myriad-Dreamin/typst.ts/pull/850
- Fixed source span resolution by falling back to `vpath` when needed.
- Added unified compiler interface in https://github.com/Myriad-Dreamin/typst.ts/pull/877
- Added PDF standards support for the web compiler in https://github.com/Myriad-Dreamin/typst.ts/pull/878
- Bumped typst to v0.15.1 in https://github.com/Myriad-Dreamin/typst.ts/pull/887
- Added `evict_cache` to the web compiler in https://github.com/Myriad-Dreamin/typst.ts/pull/895
- Avoided panic on string method calls in AST export in https://github.com/Myriad-Dreamin/typst.ts/pull/833
- Bumped tiny-skia to 0.12 and svgtypes to 0.16
- Renamed the release crates workflow file to fix the typo in https://github.com/Myriad-Dreamin/typst.ts/pull/896

**Full Changelog**: https://github.com/Myriad-Dreamin/typst.ts/compare/v0.7.0...v0.8.0

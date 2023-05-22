# Prerequisite - Download Font Assets

The font assets for Typst.ts are not included in this repository. You need to download and put them in following directories.

- To make Compiler happy, linking the font directory to `assets/fonts`.

- To make the [Renderer Sample](https://github.com/Myriad-Dreamin/typst.ts/blob/9f9295cf130092f9719d771f3969914967265f2a/renderer/src/driver/main.ts#L27-L34) happy, linking the font directory to `/pacakges/typst.ts/dist/fonts`.

There are several ways to downloading the font files:

- Download the font files from [Typst repository](https://github.com/typst/typst/tree/main/assets/fonts).

- Download font files from our [Release Page](https://github.com/Myriad-Dreamin/typst.ts/releases/tag/v0.1.0).

- Download font files using `tools/fontctl`
  ```shell
  $ cargo run --bin typst-ts-fontctl
  ```

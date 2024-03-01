# Prerequisite - Download Font Assets

The font assets for Typst.ts are not included in this repository. You need to download and put them in following directories.

- To make Compiler happy, linking the font directory to `assets/fonts`.

- To make the [Renderer Sample](https://github.com/Myriad-Dreamin/typst.ts/blob/9f9295cf130092f9719d771f3969914967265f2a/renderer/src/driver/main.ts#L27-L34) happy, linking the font directory to `/pacakges/typst.ts/dist/fonts`.

There are several ways to downloading the font files:

- Download the font files from [Typst repository](https://github.com/typst/typst/tree/main/assets/fonts).

- Download the font files via `git`:

  Please use following command inside of `typst.ts` repo:

  ```shell
  # init inside typst.ts repository
  $ git submodule update --init --recursive .
  # update inside typst.ts repository
  $ git submodule update --recursive
  ```

  Please use following command outside of `typst.ts` repo:

  ```shell
  # outside typst.ts repository
  $ git clone https://github.com/Myriad-Dreamin/typst/ fonts --single-branch --branch assets-fonts --depth 1
  ```

- Download font files from our [Release Page](https://github.com/Myriad-Dreamin/typst.ts/releases/tag/v0.1.0).

# vite-plugin-typst

A [Vite](https://vitejs.dev/) plugin for [typst](https://www.typst.app/).

## Installation

```bash
yarn add -D @myriad-dreamin/vite-plugin-typst
```

### Installing Typst Support

Two providers are expected to work:

- (Default) `@myriad-dreamin/typst-ts-node-compiler`: A js integrated compiler for typst, which makes cache shared between typst compilations.
- (Todo) `typst-cli`: Using the typst cli to compile `.typ` files.

Install the `@myriad-dreamin/typst-ts-node-compiler` package to add support for `.typ` files:

```bash
yarn add -D @myriad-dreamin/typst-ts-node-compiler
```

## Usage

The default usage to compile `index.typ` is simple, just add the plugin to your Vite config:

```ts
// vite.config.ts
import { defineConfig } from 'vite';
import typst from '@myriad-dreamin/vite-plugin-typst';

export default defineConfig({
  plugins: [typst()],
});
```

Runs `vite build` for production and runs `vite` for development. When in development, the plugin will watch for changes to `.typ` files and recompile them on the fly.

### Multiple pages

If you have multiple pages, you can specify the entry file for each page:

```ts
// vite.config.ts
export default defineConfig({
  plugins: [typst({ documents: ['content/a.typ', 'content/b.typ'] })],
});
```

Glob patterns are also supported:

```ts
// vite.config.ts
export default defineConfig({
  plugins: [typst({ documents: ['content/**/*.typ'] })],
});
```

### Sets a different root directory

By default, the root directory is the vite's configured root (`viteConfig.root`). You can set a different root directory:

```ts
// vite.config.ts
export default defineConfig({
  plugins: [typst({ root: 'typ/root/' })],
});
```

### Sets the typst provider

(Todo) By default, the plugin uses the `@myriad-dreamin/typst-ts-node-compiler` provider. You can set a different provider:

```ts
// vite.config.ts
export default defineConfig({
  plugins: [typst({ provider: 'typst-cli' })],
});
```

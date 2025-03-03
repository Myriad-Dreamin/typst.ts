# vite-plugin-typst

A [Vite](https://vitejs.dev/) plugin for [typst](https://www.typst.app/).

## Installation

```bash
yarn add -D @myriaddreamin/vite-plugin-typst
```

### Installing Typst Support

Two providers are expected to work:

- (Default) `@myriaddreamin/typst-ts-node-compiler`: A js integrated compiler for typst, which makes cache shared between typst compilations.
- (Todo) `typst-cli`: Using the typst cli to compile `.typ` files.

Install the `@myriaddreamin/typst-ts-node-compiler` package to add support for `.typ` files:

```bash
yarn add -D @myriaddreamin/typst-ts-node-compiler
```

## On-demand JS import (examples/js-import)

The default usage is simple, just add the plugin to your Vite config:

```ts
// vite.config.ts
import { defineConfig } from 'vite';
import typst from '@myriaddreamin/vite-plugin-typst';

export default defineConfig({
  plugins: [typst()],
});
```

In your `.js` or `.ts` files, you can import `.typ` files directly:

```ts
import html from 'index.typ?html';
```

Or only gets the body of the document:

```ts
import { title, description, body } from 'index.typ?parts';
```

Available query parameters:

- `html`: Get the full HTML content of the document.
- `parts`: Get the parts of the document. The parts are exported as an object with keys as the part names.
  - `body` (`string`): The body of the document.
  - `title` (`string | null`): The title of the document.
  - `description` (`string | null`): The description of the document.

Runs `vite build` for production and runs `vite` for development. When in development, the plugin will watch to `.typ` files and recompile them on changes.

## Compiling `.typ` Files into static HTML (examples/single-file)

You can also use typst documents as static HTML files. For example, compile `index.typ` into `index.html`:

```ts
// vite.config.ts
export default defineConfig({
  plugins: [typst({ index: true })],
});
```

### Multiple Pages (examples/glob-documents)

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

### Configuring a Different Root Directory

By default, the root directory is the vite's configured root (`viteConfig.root`). You can set a different root directory:

```ts
// vite.config.ts
export default defineConfig({
  plugins: [typst({ root: 'typ/root/' })],
});
```

### Configuring the Typst Compiler

(Todo) By default, the plugin uses the `@myriaddreamin/typst-ts-node-compiler` compiler. You can set a different compiler:

```ts
// vite.config.ts
export default defineConfig({
  plugins: [typst({ compiler: 'typst-cli' })],
});
```

## Customized Query (examples/mixin-parts)

You can inject query data into `?parts` query by providing a `onResolveParts` function:

```ts
import { checkExecResult } from '@myriaddreamin/vite-plugin-typst';

export default defineConfig({
  plugins: [
    TypstPlugin({
      onResolveParts: (mainFilePath, project, ctx) => {
        const res = checkExecResult(mainFilePath, project.compileHtml({ mainFilePath }), ctx);
        return {
          frontmatter: res && project.query(res, { selector: '<frontmatter>', field: 'value' })[0],
        };
      },
    }),
  ],
});
```

Then, you can import the injected data in your JavaScript files:

```ts
import { body, frontmatter } from 'main.typ?parts';

console.log(frontmatter);

document.body.innerHTML = body;
```

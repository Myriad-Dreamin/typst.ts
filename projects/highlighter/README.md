# highlighter-typst

Usage with `highlight.js`:

```html
<!-- import as cjs, js bundled, wasm bundled -->
<script
  id="script-main"
  src="https://cdn.jsdelivr.net/npm/@myriaddreamin/highlighter-typst/dist/cjs/contrib/hljs/typst.bundle.js"
></script>
<!-- import as cjs, js not bundled, wasm bundled -->
<!-- <script id="script-main" src="https://cdn.jsdelivr.net/npm/@myriaddreamin/highlighter-typst/dist/cjs/contrib/hljs/typst-lite.bundle.js"></script> -->
<!-- import as esm, js bundled, wasm bundled -->
<!-- <script id="script-main" type="module" src="https://cdn.jsdelivr.net/npm/@myriaddreamin/highlighter-typst/dist/esm/contrib/hljs/typst.bundle.js"></script> -->
<!-- import as esm, js not bundled, wasm not bundled -->
<!-- <script id="script-main" type="module" src="https://cdn.jsdelivr.net/npm/@myriaddreamin/highlighter-typst/dist/esm/contrib/hljs/typst-lite.mjs"></script> -->
<!-- import as esm, js bundled, wasm not bundled -->
<!-- <script id="script-main" type="module" src="https://cdn.jsdelivr.net/npm/@myriaddreamin/highlighter-typst/dist/esm/contrib/hljs/typst-lite.bundle.js"></script> -->

<script>
  const run = $typst$parserModule.then(() => {
    hljs.registerLanguage(
    'typst',
    window.hljsTypst({ // TypstHljsOptions
      codeBlockDefaultLanguage: 'typst',
    }),
  );
  // esm
  document.getElementById('script-main').onload = run;
  // cjs
  run();
</script>
```

Documentation for `highlight.js` apis:

````ts
/**
 * A function that constructs a language definition for hljs
 * @param options options for the hljsTypst function.
 * @returns a language definition for hljs.
 * See {@link TypstHljsOptions} for more details.
 *
 * @example
 *
 * Default usage:
 * ```ts
 * hljs.registerLanguage('typst', window.hljsTypst());
 * ```
 *
 * @example
 *
 * Don't handle code blocks:
 * ```ts
 * hljs.registerLanguage('typst', window.hljsTypst({
 *  handleCodeBlocks: false,
 * }));
 *
 * @example
 *
 * Handle code blocks with a custom function:
 * ```ts
 * hljs.registerLanguage('typst', window.hljsTypst({
 *   handleCodeBlocks: (code, emitter) => {
 *     return false;
 *   });
 * }));
 * ```
 *
 * @example
 *
 * Set the default language for code blocks:
 * ```ts
 * hljs.registerLanguage('typst', window.hljsTypst({
 *  codeBlockDefaultLanguage: 'rust',
 * }));
 * ```
 */
export function hljsTypst(options?: TypstHljsOptions);

/**
 * Options for the `hljsTypst` function.
 * @param handleCodeBlocks - Whether to handle code blocks.
 *   Defaults to true.
 *   If set to false, code blocks will be rendered as plain code blocks.
 *   If set to true, a default handler will be used.
 *   If set to a function, the function will be used as the handler.
 *
 *   When the `hljsTypst` has a code block handler, the code block will be called with the code block content and the emitter.
 *
 *   If the handler return false, the code block will be still rendered as plain code blocks.
 *
 * @param codeBlockDefaultLanguage - The default language for code blocks.
 *   Defaults to undefined.
 */
export interface TypstHljsOptions {
  handleCodeBlocks?: boolean | ((code: string, emitter: any) => /*handled*/ boolean);
  codeBlockDefaultLanguage?: string;
}
````

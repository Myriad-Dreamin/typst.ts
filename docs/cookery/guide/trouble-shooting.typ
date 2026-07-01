#import "/docs/cookery/book.typ": book-page

#show: book-page.with(title: "Trouble shooting")

= Trouble shooting

We are collecting questions and answers about the `typst.ts` project here. Feel free to ask questions in #link("https://github.com/Myriad-Dreamin/typst.ts/discussions")[Github Discussions].

== Browser cannot load the Wasm module

If the browser reports that `typst_ts_renderer_bg.wasm` or `typst_ts_web_compiler_bg.wasm` cannot be loaded, configure the module URL before the first compile or render call:

```ts
$typst.setCompilerInitOptions({
  getModule: () => '/path/to/typst_ts_web_compiler_bg.wasm',
});
$typst.setRendererInitOptions({
  getModule: () => '/path/to/typst_ts_renderer_bg.wasm',
});
```

The URL must be reachable from the web page, not just present on the local filesystem. When serving from another domain, also check CORS headers.

== Browser compilation misses fonts

Browser builds do not automatically see system fonts. Either use the default web font assets or preload the fonts your document needs before compiling:

```ts
$typst.use(
  TypstSnippet.preloadFontFromUrl('/fonts/LibertinusSerif-Regular.otf'),
);
```

For fully offline applications, serve the font files with your application and avoid CDN-only URLs.

== Node.js and browser packages behave differently

Use `@myriaddreamin/typst-ts-node-compiler` for Node.js when possible. It uses the native addon and can access local files and system fonts more naturally. Use `@myriaddreamin/typst.ts` in browsers, where Wasm module paths, font assets, and network access must be configured explicitly.

== Runtime rendering errors after upgrading

Keep `@myriaddreamin/typst.ts`, `@myriaddreamin/typst-ts-renderer`, `@myriaddreamin/typst-ts-web-compiler`, and framework wrappers on the same release line. Mixing versions can produce artifact or renderer protocol mismatches.

== The all-in-one browser bundle is too large

The full all-in-one bundle is for quick prototypes and offline demos. For production browser applications, prefer `all-in-one-lite.bundle.js` or direct package imports, then provide the compiler/renderer Wasm modules and fonts from your own asset pipeline.

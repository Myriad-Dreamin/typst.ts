# Native HTML and MathML in the web compiler

The web compiler can export Typst's native HTML document, including native
MathML equations, entirely in the browser. It compiles an HTML target and calls
Typst's HTML serializer. No SVG-to-MathML conversion or renderer is involved.

HTML export is **experimental** in the pinned Typst 0.15.1. Its output and supported
features can change; some layout features are unsupported or become embedded SVG.
See [Typst's HTML export tracking issue](https://github.com/typst/typst/issues/5512).
Equations such as fractions, roots, scripts, and matrices use native MathML.

## Build configuration

HTML serialization is opt-in through the `html` Cargo feature on
`typst-ts-web-compiler`. The normal `build`, `build:with-fonts`, and `misc` feature
set do not enable it. `full` includes it. This contribution does not introduce a
separate npm package or change the default distributed bundle.

From the repository root, using the pinned Rust toolchain, wasm-pack, and Yarn 1:

```sh
rustup target add wasm32-unknown-unknown
yarn install --frozen-lockfile
yarn --cwd packages/compiler build:html
yarn --cwd packages/renderer build
yarn --cwd packages/typst.ts build
```

`build:html` runs the following and then the existing `wasm-debundle.mjs` packaging
step, producing a matching JavaScript wrapper and WASM in `packages/compiler/pkg`:

```sh
cd packages/compiler
wasm-pack build --target web --scope myriaddreamin -- \
  --locked --no-default-features --features web,misc,html
node ../tools/wasm-debundle.mjs
```

Serve the wrapper and WASM from the same build. Existing published bundles do not
gain HTML export by updating the TypeScript wrapper alone. Without the feature,
an HTML export request rejects with a message explaining how to rebuild; PDF and
vector export remain available. The existing `world.compileHtml()` helper still
compiles without serialization in either build.

## API

```js
import { $typst } from '@myriaddreamin/typst.ts/contrib/snippet';

// Point to the HTML-enabled WASM built above, and use its matching wrapper.
$typst.setCompilerInitOptions({ getModule: () => '/typst_ts_web_compiler_bg.wasm' });
const html = await $typst.html({ mainContent: '$ (a + b) / c $' });
```

`$typst.html()` returns `Promise<string | undefined>`, like the existing snippet
export helpers, and rejects on compilation or serialization errors. It reuses the
same font loading, source files, package registry, inputs, and compiler reset as
`$typst.pdf()`. Inline sources are removed after success or failure. You can also
pass `mainFilePath`, or omit options to use `setMainFilePath()`.

For diagnostics or multiple exports from a snapshot:

```ts
import { CompileFormatEnum } from '@myriaddreamin/typst.ts/compiler';

const output = await compiler.compile({
  mainFilePath: '/main.typ',
  format: CompileFormatEnum.html,
  diagnostics: 'full',
});
// output.result: string | undefined
// output.diagnostics: structured compilation/serialization diagnostics
// output.hasError: whether HTML export failed

await compiler.runWithWorld({ mainFilePath: '/main.typ' }, async world => {
  const html = await world.html(); // { result?: string, diagnostics?, hasError? }
  const pdf = await world.pdf(); // { result?: Uint8Array, ... }
  const vector = await world.vector();
});
```

`diagnostics: 'unix'` returns textual diagnostics; `'none'` rejects on errors.
Successful HTML export retains compilation warnings. HTML compilation enables
Typst's experimental feature on the HTML task's world; it does not change the
paged target or subsequent compiler snapshots.

The format enum keeps the existing values (`vector = 0`, `pdf = 1`, `_dummy = 2`)
and adds `html = 3`. PDF/vector results remain `Uint8Array`; HTML results are
strings. Raw WASM callers can use `compiler.compile(path, inputs, 'html', diagnostics)`
or `world.get_artifact(3, diagnostics)`; diagnostics mode `0` returns a bare string,
following the raw binary export convention.

## Browser example and standalone MathML

After building, run `yarn --cwd packages/typst.ts vite --host 127.0.0.1` and open
`http://127.0.0.1:5173/examples/native-html.html`. The
[example](../packages/typst.ts/examples/native-html.html) loads local font assets,
compiles source in WASM, and shows both the HTML and extracted MathML.

```js
const doc = new DOMParser().parseFromString(html, 'text/html');
const math = [...doc.getElementsByTagNameNS('http://www.w3.org/1998/Math/MathML', 'math')];
const mathml = math.map(element => new XMLSerializer().serializeToString(element));
```

The HTML parser assigns the MathML namespace. `XMLSerializer` adds the `xmlns`
declaration when extracting standalone XML fragments. Use the complete HTML for
Typst's document styles; isolated MathML can require those styles and suitable
fonts to reproduce the document's appearance.

## Verification and size comparison

Run both configurations; the browser tests explicitly select which capability
to expect. The regression suite also checks the existing compiler exports in
Node and Chromium.

```sh
yarn --cwd packages/compiler build
yarn --cwd packages/typst.ts test src/compiler-html.browser.test.mts src/compiler-cleanup.all.test.mts src/compiler.all.test.mts
yarn --cwd packages/compiler build:html
VITE_TYPST_HTML=1 yarn --cwd packages/typst.ts test src/compiler-html.browser.test.mts src/compiler-cleanup.all.test.mts src/compiler.all.test.mts
```

In PowerShell, set `$env:VITE_TYPST_HTML = '1'` before the second test run. CI runs
the default configuration before rebuilding and testing the HTML configuration.

For a size comparison, save `packages/compiler/pkg/typst_ts_web_compiler_bg.wasm`
after each release build, then run:

```sh
node packages/compiler/scripts/measure-wasm-size.mjs default.wasm html.wasm
```

The script compares uncompressed bytes and gzip level 9 using Node's zlib. Use the
same Rust, wasm-pack, wasm-opt, and font features for both builds.

Measured on Windows x64 with Rust 1.92.0, wasm-pack 0.15.0, wasm-bindgen 0.2.118,
wasm-opt 117 (the repository's `-O` settings), and Node 24.16.0. Both are release
builds without embedded fonts:

| Build           |      WASM bytes |  gzip -9 bytes |
| --------------- | --------------: | -------------: |
| `web,misc`      |      30,280,215 |     11,056,578 |
| `web,misc,html` |      30,386,380 |     11,100,567 |
| Increase        | 106,165 (0.35%) | 43,989 (0.40%) |

The existing compiler already contains HTML compilation helpers, so this measures
the incremental cost of exposing native HTML serialization. Fonts and JavaScript
wrappers are excluded; toolchain or dependency changes can change the totals.

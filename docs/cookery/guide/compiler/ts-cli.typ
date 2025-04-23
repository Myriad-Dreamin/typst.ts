#import "/docs/cookery/book.typ": book-page
#import "/docs/cookery/term.typ" as term

#show: book-page.with(title: "Command Line Interface")

#include "../claim.typ"

The unique feature of `typst-ts-cli` is that it precompiles typst documents into #term.vector-format files. It internally runs #link("https://github.com/typst/typst")[Typst compiler] with `typst.ts`'s exporters.

```ts
// The './main.sir.in' could be obtained by `typst-ts-cli`
//   Specifically, by `compile ... --format vector`
const vectorData: Uint8Array = await
  fetch('./main.sir.in').into();

// into svg format
await $typst.svg({ vectorData });
// into canvas operations
await $typst.canvas(div, { vectorData });
```

For more usage of #term.vector-format files, please refer to #link("https://myriad-dreamin.github.io/typst.ts/cookery/guide/renderers.html")[Renderers] section.

== Installation

Install latest CLI of typst.ts via cargo:

```bash
cargo install --locked --git https://github.com/Myriad-Dreamin/typst.ts typst-ts-cli
```

Or Download the latest release from #link("https://github.com/Myriad-Dreamin/typst.ts/releases")[GitHub Releases].

== The compile command

=== Typical usage

compile a document to a PDF file and a #term.vector-format file.

```bash
typst-ts-cli compile \
  --workspace "fuzzers/corpora/math" \
  --entry "fuzzers/corpora/math/main.typ"
```

=== `-e,--entry` option, required

Entry file.

```bash
typst-ts-cli -e main.typ
```

=== `-w,--workspace` option, default: `.`

Path to typst workspace.

```bash
typst-ts-cli -w /repos/root/ -e main.typ
```

=== `--watch` option

Watch file dependencies and compile the document.

```bash
typst-ts-cli compile ... --watch
```

=== `--format` option

compile a document to specific formats.

```bash
# export nothing (dry compile)
typst-ts-cli compile ... --format nothing
# into vector format
typst-ts-cli compile ... --format vector
# into multiple formats at the same time
typst-ts-cli compile ... --format svg --format svg_html
```

=== `--dynamic-layout` option

Please setup the #link("https://github.com/Myriad-Dreamin/typst.ts/tree/main/contrib/templates/variables")[variables] package before compilation.

```bash
typst-ts-cli package link --manifest \
  repos/typst.ts/contrib/templates/variables/typst.toml
typst-ts-cli compile ... --dynamic-layout
```

=== `-o,--output` option

Output to directory, default in the same directory as the entry file.

```bash
typst-ts-cli compile ... -o dist
```

=== `--trace` option

Comma separated options to trace execution of typst compiler when compiling documents:

```bash
# trace events at warning level
typst-ts-cli compile ... --trace=verbosity=0
# trace events at info level
typst-ts-cli compile ... --trace=verbosity=1
# trace events at debug level
typst-ts-cli compile ... --trace=verbosity=2
# trace events at trace level
typst-ts-cli compile ... --trace=verbosity=3
```

=== Example: compile a document with watching dependencies

```bash
typst-ts-cli compile \
  -e "fuzzers/corpora/math/main.typ"
  --watch
```

=== Example: compile a document into SVG

```bash
typst-ts-cli compile \
  -e "fuzzers/corpora/math/main.typ"
  --format svg
```

=== Example: compile a document into SVG wrapped with HTML

```bash
typst-ts-cli compile \
  -e "fuzzers/corpora/math/main.typ"
  --format svg_html
```

=== Example: compile a document into the #term.vector-format

```bash
typst-ts-cli compile \
  -e "fuzzers/corpora/math/main.typ"
  --format vector
```

=== Example: compile a document into the #term.vector-format containing responsive layouts

```bash
typst-ts-cli compile \
  -e "fuzzers/corpora/math/main.typ"
  --dynamic-layout
```

// == Package commands

// === Example: list packages in `@preview` namespace

// ```bash
// typst-ts-cli package list
// ```

// === Example: link a package to `@preview` namespace

// ```bash
// typst-ts-cli package link --manifest path/to/typst.toml
// ```

// === Example: unlink a package from `@preview` namespace

// ```bash
// typst-ts-cli package unlink --manifest path/to/typst.toml
// ```

== CLI Options

Help:

```bash
$ typst-ts-cli --help
The cli for typst.ts.

Usage: typst-ts-cli [OPTIONS] [COMMAND]

Commands:
  compile  Run compiler. [aliases: c]
  completion  Generate shell completion script
  env      Dump Client Environment.
  font     Commands about font for typst.
  help     Print this message or the help of the given subcommand(s)
  package     Commands about package for typst.

Options:
  -V, --version  Print Version
      --VV <VV>  Print Version in format [default: none] [possible values: none, short, features, full, json, json-plain]
  -h, --help     Print help
```

#import "/docs/cookery/book.typ": book-page

#show: book-page.with(title: "Command Line Interface")

= Command Line Interface

Run #link("https://github.com/typst/typst")[Typst compiler] with `typst.ts`'s exporters (renderers) Example:

```bash
# Optional
typst-ts-cli compile \
  --workspace "fuzzers/corpora/math" \
  --entry "fuzzers/corpora/math/main.typ"
```

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
      --VV <VV>  Print Version in format [default: none] [possible values: none, short, full, json, json-plain]
  -h, --help     Print help
```

Package Help:

```bash
$ typst-ts-cli package --help
Commands about package for typst.

Usage: typst-ts-cli package <COMMAND>

Commands:
  doc     Generate documentation for a package
  help    Print this message or the help of the given subcommand(s)
  link    Link a package to local data path
  list    List all discovered packages in data and cache paths
  unlink  Unlink a package from local data path

Options:
  -h, --help  Print help
```

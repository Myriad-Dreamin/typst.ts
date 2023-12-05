#import "/docs/cookery/book.typ": book-page

#show: book-page.with(title: "Hexo Library")

= Hexo Library

== Install dependencies

Note: please align the version of the dependencies to same vesrion, otherwise you may get runtime rendering error.

Add `"@myriaddreamin/typst-ts-renderer": "^0.4.1"` to your `package.json`.

And download `typst-ts-cli` from #link("https://github.com/Myriad-Dreamin/typst.ts/releases/tag/v0.4.1")[GitHub Release].

Verify the vesrion of cli:

```bash
$ typst-ts-cli --version
typst-ts-cli version 0.4.1
features: ....
```

== Usage

Add `"hexo-renderer-typst": "^0.4.1"` to your `package.json`

And run:

```
# serve files
hexo serve
# generate files
hexo generate
```

Currently, it could only render typst documents inside of `source/_posts` (Hexo Posts) and fix typst workspace (root directory) to the root of your blog project.

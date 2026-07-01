#import "/docs/cookery/book.typ": book-page, cross-link
#import "/docs/cookery/term.typ" as term

#show: book-page.with(title: "Start it Easily")
#import "/docs/cookery/book.typ": *

We provide all-in-one libraries to simplify setup for #cross-link("/guide/all-in-one.typ")[browsers] and #cross-link("/guide/all-in-one-node.typ")[Node.js].

"All-in-one" means the first API hides most compiler and renderer setup. This is convenient while you are prototyping. In browsers, move from the full bundle to the lite bundle or direct package imports when bundle size, font loading, or Wasm module hosting becomes important.

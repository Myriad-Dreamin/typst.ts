#import "@preview/shiroa:0.1.2": *

#import "/docs/cookery/templates/ebook.typ"

#show: ebook.project.with(title: "reflexo-typst", spec: "book.typ")

// set a resolver for inclusion
#ebook.resolve-inclusion(it => include it)

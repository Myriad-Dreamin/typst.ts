#import "@preview/book:0.2.5": *

#import "/docs/cookery/templates/ebook.typ"

#show: ebook.project.with(title: "typst-book", spec: "book.typ")

// set a resolver for inclusion
#ebook.resolve-inclusion(it => include it)

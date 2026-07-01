#import "/docs/cookery/book.typ": book-page, cross-link
#import "/docs/cookery/term.typ" as term

#show: book-page.with(title: "Start it Easily")
#import "/docs/cookery/book.typ": *

We provide all-in-one library to simplify build up, for #cross-link("/guide/all-in-one.typ")[browsers] and #cross-link("/guide/compiler/service.typ")[Node.js.]

"All-in-one" means that all resources are built in one library, regardless whether you need them or not. The resources include the compiler module, the renderer module, the fonts, and convenient scripts. This is not a problem in Node.js, but it is a problem in browsers because of the bundle size. However, you can go back and optimize them after you finishing building your applications.

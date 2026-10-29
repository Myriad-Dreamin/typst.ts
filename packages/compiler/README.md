# typst-ts-web-compiler

The compiler can run in browser. See documentation for details:

- [Get Started](https://myriad-dreamin.github.io/typst.ts/cookery/get-started.html)
- [Compiler interfaces](https://myriad-dreamin.github.io/typst.ts/cookery/guide/compilers.html)

It can also runs in node.js, but limits its access to operating system. Therefore, I'd suggest to use [typst.node](https://github.com/Myriad-Dreamin/typst.ts/tree/main/packages/typst.node) for fully accessing to operating system.

## Source and preview synchronization

An incremental compiler session retains the paged document and compiler world
from its latest successful compilation. The high-level `IncrementalServer` API
can resolve a UTF-8 source byte offset to page-space positions with
`sourceToDocument`, and a page-space point back to a source byte offset with
`documentToSource`. `mappingRevision` lets a renderer associate query results
with the vector artifact it has applied.

These queries inspect Typst frames directly and do not require an HTML semantic
overlay. Coordinates use Typst points and page offsets are zero-based.

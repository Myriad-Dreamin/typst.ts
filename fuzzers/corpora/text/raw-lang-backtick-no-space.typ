
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// The language tag stops at a backtick even without whitespace.
// TODO: Do we want this behavior? It was not discussed in #7337.
#let raw = ```lang`test ` ```
#test(raw.lang, "lang")
#test(raw.text, "`test `")
#test(raw.block, false)
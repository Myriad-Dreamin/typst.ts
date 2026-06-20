
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// The language tag stops at a space.
#let raw = ```lang test ```
#test(raw.lang, "lang")
#test(raw.text, "test ")
#test(raw.block, false)
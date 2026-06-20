
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#let raw = ```lang ` ```
#test(raw.lang, "lang")
#test(raw.text, "`")
#test(raw.block, false)

#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#let raw = ``` ` ```
#test(raw.text, "`")
#test(raw.block, false)
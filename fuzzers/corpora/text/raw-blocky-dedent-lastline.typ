
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#let raw = ```
  test
 ```
#test(raw.text, " test")
#test(raw.block, true)

#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// When there is content in the first line, we discard a single whitespace char.
#let raw = ``` test
```
#test(raw.text, "test")
#test(raw.block, true)
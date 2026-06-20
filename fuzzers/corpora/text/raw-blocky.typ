
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// The first line and the last line are ignored.
#let raw = {
```
test
```
}
#test(raw.text, "test")
#test(raw.block, true)
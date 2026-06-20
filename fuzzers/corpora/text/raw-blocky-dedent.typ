
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// A blocky raw should handle dedents.
#let raw = {
```
test
```
}
#test(raw.text, "test")
#test(raw.block, true)
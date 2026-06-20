
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// The first line is not affected by dedent, and the middle lines don't consider
// the whitespace prefix of the first line.
#let raw = ``` test
     test2
  ```
#test(raw.text, "test\n   test2")
#test(raw.block, true)
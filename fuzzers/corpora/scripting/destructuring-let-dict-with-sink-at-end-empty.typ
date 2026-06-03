
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Destructuring with an empty sink.
#let (a: _, ..b) = (a: 1)
#test(b, (:))
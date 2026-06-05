
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Destructuring with an empty sink.
#let (..a, b, c) = (1, 2)
#test(a, ())
#test(b, 1)
#test(c, 2)
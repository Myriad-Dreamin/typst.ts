
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Destructuring with an empty sink.
#let (a, b, ..c) = (1, 2)
#test(a, 1)
#test(b, 2)
#test(c, ())
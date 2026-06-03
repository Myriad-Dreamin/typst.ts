
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Simple destructuring.
#let (a, b) = (1, 2)
#test(a, 1)
#test(b, 2)
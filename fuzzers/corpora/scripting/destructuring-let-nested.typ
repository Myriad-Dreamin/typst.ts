
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Nested destructuring.
#let ((a, b), (key: c)) = ((1, 2), (key: 3))
#test((a, b, c), (1, 2, 3))
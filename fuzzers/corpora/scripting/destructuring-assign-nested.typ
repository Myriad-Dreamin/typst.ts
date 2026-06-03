
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test nested destructuring assignment.
#let a
#let b
#let c
#(((a, b), (key: c)) = ((1, 2), (key: 3)))
#test((a, b, c), (1, 2, 3))
// SKIP: Temporarily removed for Typst 0.15.0-rc1 corpus compatibility review.

#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test sizing of both relative and absolute non math content in math sizes.
#let stuff = square(inset: 0pt)[hello]
#let square = square(size: 5pt)
$ stuff sum^stuff_square square $
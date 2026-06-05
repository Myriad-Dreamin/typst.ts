
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Sum doesn't get wrapped in math as it is a single expr.
// Ideally the height would match the actual height of the sum.
#let height(x) = context measure(x).height
$ sum != height(sum) $
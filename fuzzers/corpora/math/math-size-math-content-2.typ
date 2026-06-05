
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Nested math content has styles overwritten by the inner equation.
// Ideally the heights would match the actual height of the sums.
#let sum = $sum^2$
#let height(x) = context measure(x).height
$sum = height(sum) $
$ sum != height(sum) $
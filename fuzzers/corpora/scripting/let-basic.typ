
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Automatically initialized with none.
#let x
#test(x, none)

// Manually initialized with one.
#let z = 1
#test(z, 1)

// Syntax sugar for function definitions.
#let fill = conifer
#let f(body) = rect(width: 2cm, fill: fill, inset: 5pt, body)
#f[Hi!]
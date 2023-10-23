
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Evaluating a math expr should renders the same as an equation

#eval(mode: "math", "f(a) = cases(a + b\, space space x >= 3,a + b\, space space x = 5)")

$f(a) = cases(a + b\, space space x >= 3,a + b\, space space x = 5)$

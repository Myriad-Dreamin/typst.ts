
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test the `erf` function.
#test(calc.erf(0), 0)
#test(calc.erf(1), 0.8427007929497149)
#test(calc.erf(calc.inf), 1)
#test(calc.erf(-1), -calc.erf(1))
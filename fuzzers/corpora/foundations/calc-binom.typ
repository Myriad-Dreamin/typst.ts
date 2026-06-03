
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test the `binom` function.
#test(calc.binom(0, 0), 1)
#test(calc.binom(5, 3), 10)
#test(calc.binom(5, 5), 1)
#test(calc.binom(5, 6), 0)
#test(calc.binom(6, 2), 15)
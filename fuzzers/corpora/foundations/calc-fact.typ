
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test the `fact` function.
#test(calc.fact(0), 1)
#test(calc.fact(5), 120)
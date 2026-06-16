
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test the `quo` function.
#test(calc.quo(1, 1), 1)
#test(calc.quo(5, 3), 1)
#test(calc.quo(5, -3), -1)
#test(calc.quo(22.5, 10), 2)
#test(calc.quo(9, 4.5), 2)
#test(calc.quo(decimal("22.5"), 10), 2)
#test(calc.quo(decimal("9"), decimal("4.5")), 2)

#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test the `perm` function.
#test(calc.perm(0, 0), 1)
#test(calc.perm(5, 3), 60)
#test(calc.perm(5, 5), 120)
#test(calc.perm(5, 6), 0)
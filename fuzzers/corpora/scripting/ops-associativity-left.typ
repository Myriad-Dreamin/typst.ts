
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Math operators are left-associative.
#test(10 / 2 / 2 == (10 / 2) / 2, true)
#test(10 / 2 / 2 == 10 / (2 / 2), false)
#test(1 / 2 * 3, 1.5)

#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test the `gcd` function.
#test(calc.gcd(112, 77), 7)
#test(calc.gcd(12, 96), 12)
#test(calc.gcd(13, 9), 1)
#test(calc.gcd(13, -9), 1)
#test(calc.gcd(272557, 272557), 272557)
#test(calc.gcd(0, 0), 0)
#test(calc.gcd(7, 0), 7)

#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test the `pow`, `log`, `exp`, and `ln` functions.
#test(calc.pow(10, 0), 1)
#test(calc.pow(2, 4), 16)
#test(calc.pow(decimal("0.5"), 18), decimal("0.000003814697265625"))
#test(calc.exp(2), calc.pow(calc.e, 2))
#test(calc.ln(10), calc.log(10, base: calc.e))
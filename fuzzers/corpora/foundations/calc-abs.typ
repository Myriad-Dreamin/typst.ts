
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test the `abs` function.
#test(calc.abs(-3), 3)
#test(calc.abs(3), 3)
#test(calc.abs(-0.0), 0.0)
#test(calc.abs(0.0), -0.0)
#test(calc.abs(-3.14), 3.14)
#test(calc.abs(50%), 50%)
#test(calc.abs(-25%), 25%)
#test(calc.abs(decimal("4932.493249324932")), decimal("4932.493249324932"))
#test(calc.abs(decimal("-12402.593295932041")), decimal("12402.593295932041"))
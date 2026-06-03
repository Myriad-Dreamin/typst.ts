
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test the `rem` function.
#test(calc.rem(1, 1), 0)
#test(calc.rem(5, 3), 2)
#test(calc.rem(5, -3), 2)
#test(calc.rem(22.5, 10), 2.5)
#test(calc.rem(9, 4.5), 0)
#test(calc.rem(decimal("5"), -3), decimal("2"))
#test(calc.rem(decimal("22.5"), decimal("10")), decimal("2.5"))
#test(calc.rem(9, decimal("4.5")), decimal("0"))
#test(calc.rem(decimal("7"), decimal("3")), decimal("1"))
#test(calc.rem(decimal("7"), decimal("-3")), decimal("1"))
#test(calc.rem(decimal("-7"), decimal("3")), decimal("-1"))
#test(calc.rem(decimal("-7"), decimal("-3")), decimal("-1"))

// Ensure `i64::MIN % -1` will not overflow and panic.
#test(calc.rem(int("-9223372036854775808"), -1), 0)
#test(calc.rem(float("-9223372036854775808"), -1.0), 0.0)
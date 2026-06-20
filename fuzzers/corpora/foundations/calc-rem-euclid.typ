
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test the `rem-euclid` function.
#test(calc.rem-euclid(7, 3), 1)
#test(calc.rem-euclid(7, -3), 1)
#test(calc.rem-euclid(-7, 3), 2)
#test(calc.rem-euclid(-7, -3), 2)
#test(calc.rem-euclid(2.5, 2), 0.5)
#test(calc.rem-euclid(decimal("7"), decimal("3")), decimal("1"))
#test(calc.rem-euclid(decimal("7"), decimal("-3")), decimal("1"))
#test(calc.rem-euclid(decimal("-7"), decimal("3")), decimal("2"))
#test(calc.rem-euclid(decimal("-7"), decimal("-3")), decimal("2"))
#test(calc.rem-euclid(decimal("2.5"), decimal("2")), decimal("0.5"))

// Ensure `i64::MIN % -1` will not overflow and panic.
#test(calc.rem-euclid(int.min, -1), 0)
#test(calc.rem-euclid(float(int.min), -1.0), 0.0)
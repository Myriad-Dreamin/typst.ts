
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test the `min` and `max` functions.
#test(calc.min(2, -4), -4)
#test(calc.min(3.5, 1e2, -0.1, 3), -0.1)
#test(calc.min(decimal("3.5"), 4, decimal("-3213.99999")), decimal("-3213.99999"))
#test(calc.max(-3, 11), 11)
#test(calc.max(decimal("3"), 45), 45)
#test(calc.min("hi"), "hi")
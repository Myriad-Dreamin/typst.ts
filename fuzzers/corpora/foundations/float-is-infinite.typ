
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test float `is-infinite()`.
#test(float(calc.inf).is-infinite(), true)
#test(float(-calc.inf).is-infinite(), true)
#test(float(10).is-infinite(), false)
#test(float(-10).is-infinite(), false)
#test(float(float.nan).is-infinite(), false)
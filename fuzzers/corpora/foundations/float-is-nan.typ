
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test float `is-nan()`.
#test(float(float.nan).is-nan(), true)
#test(float(10).is-nan(), false)
#test(float(calc.inf).is-nan(), false)
#test(float(-calc.inf).is-nan(), false)
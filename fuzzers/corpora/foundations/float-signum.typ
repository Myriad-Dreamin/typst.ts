
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test float `signum()`
#test(float(0.0).signum(), 1.0)
#test(float(1.0).signum(), 1.0)
#test(float(-1.0).signum(), -1.0)
#test(float(10.0).signum(), 1.0)
#test(float(-10.0).signum(), -1.0)
#test(float(calc.inf).signum(), 1.0)
#test(float(-calc.inf).signum(), -1.0)
#test(float(float.nan).signum().is-nan(), true)
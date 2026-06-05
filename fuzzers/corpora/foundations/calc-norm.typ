
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#test(calc.norm(1, 2, -3, 0.5), calc.sqrt(14.25))
#test(calc.norm(3, 4), 5.0)
#test(calc.norm(3, 4), 5.0)
#test(calc.norm(), 0.0)
#test(calc.norm(p: 3, 1, -2), calc.pow(9, 1/3))
#test(calc.norm(p: calc.inf, 1, -2), 2.0)
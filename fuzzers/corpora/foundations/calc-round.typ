
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#test(calc.round(calc.e, digits: 2), 2.72)
#test(calc.round(calc.pi, digits: 2), 3.14)
#test(type(calc.round(3.1415, digits: 2)), float)
#test(type(calc.round(5, digits: 2)), int)
#test(type(calc.round(decimal("3.1415"), digits: 2)), decimal)
#test(type(calc.round(314.15, digits: -2)), float)
#test(type(calc.round(523, digits: -2)), int)
#test(type(calc.round(decimal("314.15"), digits: -2)), decimal)
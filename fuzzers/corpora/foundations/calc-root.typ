
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#test(calc.root(12.0, 1), 12.0)
#test(calc.root(9.0, 2), 3.0)
#test(calc.root(27.0, 3), 3.0)
#test(calc.root(-27.0, 3), -3.0)
// 100^(-1/2) = (100^(1/2))^-1 = 1/sqrt(100)
#test(calc.root(100.0, -2), 0.1)
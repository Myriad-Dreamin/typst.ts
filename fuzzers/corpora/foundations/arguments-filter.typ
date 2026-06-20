
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test the `filter` method.
#test(arguments().filter(calc.even), arguments())
#test(arguments(1, a: 2, b: 3, 4).filter(calc.even), arguments(a: 2, 4))
#test(arguments(h: 7, e: 3, l: 2, o: 5, 1).filter(x => x < 5), arguments(e: 3, l: 2, 1))
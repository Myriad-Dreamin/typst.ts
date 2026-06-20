
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test the `filter` method.
#test((:).filter(calc.even), (:))
#test((a: 0, b: 1, c: 2).filter(v => v != 0), (b: 1, c: 2))
#test((a: 0, b: 1, c: 2).filter(calc.even), (a: 0, c: 2))
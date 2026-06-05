
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test color kind method.
#test(rgb(1, 2, 3, 4).space(), rgb)
#test(cmyk(4%, 5%, 6%, 7%).space(), cmyk)
#test(luma(40).space(), luma)
#test(rgb(1, 2, 3, 4).space() != luma, true)
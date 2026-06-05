
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test the `windows` method.
#test(().windows(5), ())
#test((1, 2, 3).windows(5), ())
#test((1, 2, 3, 4, 5).windows(3), ((1, 2, 3), (2, 3, 4), (3, 4, 5)))
#test((1, 2, 3, 4, 5, 6, 7, 8).windows(5), ((1, 2, 3, 4, 5), (2, 3, 4, 5, 6), (3, 4, 5, 6, 7), (4, 5, 6, 7, 8)))
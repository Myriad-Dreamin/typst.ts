
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test the `chunks` method.
#test(().chunks(10), ())
#test((1, 2, 3).chunks(10), ((1, 2, 3),))
#test((1, 2, 3, 4, 5, 6).chunks(3), ((1, 2, 3), (4, 5, 6)))
#test((1, 2, 3, 4, 5, 6, 7, 8).chunks(3), ((1, 2, 3), (4, 5, 6), (7, 8)))

#test(().chunks(10, exact: true), ())
#test((1, 2, 3).chunks(10, exact: true), ())
#test((1, 2, 3, 4, 5, 6).chunks(3, exact: true), ((1, 2, 3), (4, 5, 6)))
#test((1, 2, 3, 4, 5, 6, 7, 8).chunks(3, exact: true), ((1, 2, 3), (4, 5, 6)))

#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test the `enumerate` method.
#test(().enumerate(), ())
#test(().enumerate(start: 5), ())
#test(("a", "b", "c").enumerate(), ((0, "a"), (1, "b"), (2, "c")))
#test(("a", "b", "c").enumerate(start: 1), ((1, "a"), (2, "b"), (3, "c")))
#test(("a", "b", "c").enumerate(start: 42), ((42, "a"), (43, "b"), (44, "c")))
#test(("a", "b", "c").enumerate(start: -7), ((-7, "a"), (-6, "b"), (-5, "c")))
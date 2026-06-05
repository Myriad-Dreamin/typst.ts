
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test the `map` method.
#test(().map(x => x * 2), ())
#test((2, 3).map(x => x * 2), (4, 6))
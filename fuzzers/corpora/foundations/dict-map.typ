
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test the `map` method.
#test(().map(x => x * 2), ())
#test((a: 2, b: 3).map(x => x * 2), (a: 4, b: 6))
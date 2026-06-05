
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test the `reduce` method.
#test(().reduce(grid), none)
#test((1, 2, 3, 4).reduce((s, x) => s + x), 10)
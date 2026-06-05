
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test the `fold` method.
#test(().fold("hi", grid), "hi")
#test((1, 2, 3, 4).fold(0, (s, x) => s + x), 10)
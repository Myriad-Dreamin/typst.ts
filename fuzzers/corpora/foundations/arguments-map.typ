
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test the `map` method.
#test(arguments().map(x => x * 2), arguments())
#test(arguments(2, a: 3).map(x => x * 2), arguments(4, a: 6))
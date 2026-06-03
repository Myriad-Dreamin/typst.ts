
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test the `dedup` method with the `key` argument.
#test((1, 2, 3, 4, 5, 6).dedup(key: x => calc.rem(x, 2)), (1, 2))
#test((1, 2, 3, 4, 5, 6).dedup(key: x => calc.rem(x, 3)), (1, 2, 3))
#test(("Hello", "World", "Hi", "There").dedup(key: x => x.len()), ("Hello", "Hi"))
#test(("Hello", "World", "Hi", "There").dedup(key: x => x.at(0)), ("Hello", "World", "There"))
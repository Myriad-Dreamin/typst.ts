
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test addition and joining.
#test(bytes((1, 2)) + bytes(()), bytes((1, 2)))
#test(bytes((1, 2)) + bytes((3, 4)), bytes((1, 2, 3, 4)))
#test(bytes(()) + bytes((3, 4)), bytes((3, 4)))
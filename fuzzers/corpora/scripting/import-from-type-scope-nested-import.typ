
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test importing from a type's scope with nested import.
#import std: array.zip, array.pop as renamed-pop
#test(zip((1, 2), (3, 4)), ((1, 3), (2, 4)))
#test(renamed-pop((1, 2)), 2)
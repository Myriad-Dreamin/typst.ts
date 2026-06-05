
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test importing from a type's scope.
#import array: zip
#test(zip((1, 2), (3, 4)), ((1, 3), (2, 4)))
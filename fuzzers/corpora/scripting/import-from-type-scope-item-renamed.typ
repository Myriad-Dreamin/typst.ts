
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test importing from a type's scope with renaming.
#import array: pop as renamed-pop
#test(renamed-pop((1, 2)), 2)
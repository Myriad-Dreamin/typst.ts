
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test the `len` method.
#test(().len(), 0)
#test(("A", "B", "C").len(), 3)
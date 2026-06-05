
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test the `rev` method.
#test(range(3).rev(), (2, 1, 0))
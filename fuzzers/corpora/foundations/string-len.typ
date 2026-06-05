
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test the `len` method.
#test("Hello World!".len(), 12)
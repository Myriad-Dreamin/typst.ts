
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test `at`'s 'default' parameter.
#test("z", "Hello".at(5, default: "z"))
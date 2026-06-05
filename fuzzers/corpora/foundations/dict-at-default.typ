
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test default value.
#test((a: 1, b: 2).at("b", default: 3), 2)
#test((a: 1, b: 2).at("c", default: 3), 3)
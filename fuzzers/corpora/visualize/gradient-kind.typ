
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test gradient functions.
#test(gradient.linear(red, green, blue).kind(), gradient.linear)
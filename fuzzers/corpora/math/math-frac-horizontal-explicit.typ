
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test that explicit fractions don't change parentheses
#set math.frac(style: "horizontal")
$ frac(a, (b + c)), frac(a, b + c) $

#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test that parentheses are in a left-right pair even when rebuilt by a horizontal fraction
#set math.frac(style: "horizontal")
$ (#v(2em)) / n $
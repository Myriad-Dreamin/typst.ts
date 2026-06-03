
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test that non-parentheses left-right pairs remain untouched
#set math.frac(style: "horizontal")
$ [x+y] / {z} $
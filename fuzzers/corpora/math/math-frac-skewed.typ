
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test skewed fractions
#set math.frac(style: "skewed")
$ a / b,  a / (b / c) $

#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test on underover
#show math.equation: set text(fill: gradient.linear(..color.map.rainbow))
#show math.equation: box

$ underline(X^2) $
$ overline("hello, world!") $

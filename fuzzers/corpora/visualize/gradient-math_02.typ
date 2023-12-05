
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test on root
#show math.equation: set text(fill: gradient.linear(..color.map.rainbow))
#show math.equation: box

$ x_"1,2" = frac(-b +- sqrt(b^2 - 4 a c), 2 a) $

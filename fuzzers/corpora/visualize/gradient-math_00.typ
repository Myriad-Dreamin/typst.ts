
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test on cancel
#show math.equation: set text(fill: gradient.linear(..color.map.rainbow))
#show math.equation: box

$ a dot cancel(5) = cancel(25) 5 x + cancel(5) 1 $

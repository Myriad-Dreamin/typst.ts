
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test on frac
#show math.equation: set text(fill: gradient.linear(..color.map.rainbow))
#show math.equation: box

$ nabla dot bold(E) = frac(rho, epsilon_0) $

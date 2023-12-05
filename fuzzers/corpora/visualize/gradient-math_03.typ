
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test on matrix
#show math.equation: set text(fill: gradient.linear(..color.map.rainbow))
#show math.equation: box

$ A = mat(
  1, 2, 3;
  4, 5, 6;
  7, 8, 9
) $

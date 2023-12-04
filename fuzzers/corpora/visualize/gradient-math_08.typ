
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test conic gradient
#show math.equation: set text(fill: gradient.conic(red, blue, angle: 45deg))
#show math.equation: box

$ A = mat(
  1, 2, 3;
  4, 5, 6;
  7, 8, 9
) $

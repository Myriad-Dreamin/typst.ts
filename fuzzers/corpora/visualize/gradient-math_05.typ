
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test a different direction
#show math.equation: set text(fill: gradient.linear(..color.map.rainbow, dir: ttb))
#show math.equation: box

$ A = mat(
  1, 2, 3;
  4, 5, 6;
  7, 8, 9
) $

$ x_"1,2" = frac(-b +- sqrt(b^2 - 4 a c), 2 a) $


#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test radial gradient
#show math.equation: set text(fill: gradient.radial(..color.map.rainbow, center: (30%, 30%)))
#show math.equation: box

$ A = mat(
  1, 2, 3;
  4, 5, 6;
  7, 8, 9
) $

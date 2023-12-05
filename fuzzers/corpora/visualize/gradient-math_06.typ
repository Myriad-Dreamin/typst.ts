
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test miscelaneous
#show math.equation: set text(fill: gradient.linear(..color.map.rainbow))
#show math.equation: box

$ hat(x) = bar x bar = vec(x, y, z) = tilde(x) = dot(x) $
$ x prime = vec(1, 2, delim: "[") $
$ sum_(i in NN) 1 + i $
$ attach(
  Pi, t: alpha, b: beta,
  tl: 1, tr: 2+3, bl: 4+5, br: 6,
) $

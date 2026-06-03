
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// In this bug, the alignment set with "show math.equation: set align(...)"
// overrides the left-right alternating behavior of alignment points.
#let equations = [
$ a + b &= c \
      e &= f + g + h $
$         a &= b + c \
  e + f + g &= h $
]
#equations

#show math.equation: set align(start)
#equations

#show math.equation: set align(end)
#equations
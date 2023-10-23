
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

#square(
  size: 50pt,
  fill: gradient.conic(..color.map.rainbow, space: color.hsv, center: (10%, 10%)),
)

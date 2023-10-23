
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page


#square(
  size: 50pt,
  fill: gradient.radial(..color.map.rainbow, space: color.hsl, radius: 10%),
)
#square(
  size: 50pt,
  fill: gradient.radial(..color.map.rainbow, space: color.hsl, radius: 72%),
)

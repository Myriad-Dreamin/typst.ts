
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

#rect(
  height: 40pt,
  width: 100%,
  fill: gradient.linear(..color.map.rainbow).sharp(10).repeat(5, mirror: false)
)

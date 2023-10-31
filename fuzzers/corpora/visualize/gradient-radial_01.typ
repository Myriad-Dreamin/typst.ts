
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page


#grid(
  columns: 2,
  square(
    size: 50pt,
    fill: gradient.radial(..color.map.rainbow, space: color.hsl, center: (0%, 0%)),
  ),
  square(
    size: 50pt,
    fill: gradient.radial(..color.map.rainbow, space: color.hsl, center: (0%, 100%)),
  ),
  square(
    size: 50pt,
    fill: gradient.radial(..color.map.rainbow, space: color.hsl, center: (100%, 0%)),
  ),
  square(
    size: 50pt,
    fill: gradient.radial(..color.map.rainbow, space: color.hsl, center: (100%, 100%)),
  ),
)

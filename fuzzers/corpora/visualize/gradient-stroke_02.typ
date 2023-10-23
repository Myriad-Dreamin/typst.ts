
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

#align(
  center + bottom,
  square(
    size: 50pt,
    fill: black,
    stroke: 10pt + gradient.conic(red, blue)
  )
)

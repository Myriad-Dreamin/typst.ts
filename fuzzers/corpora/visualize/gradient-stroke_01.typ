
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

#align(
  center + bottom,
  square(
    size: 50pt,
    fill: gradient.radial(red, blue, radius: 70.7%, focal-center: (10%, 10%)),
    stroke: 10pt + gradient.radial(red, blue, radius: 70.7%, focal-center: (10%, 10%))
  )
)

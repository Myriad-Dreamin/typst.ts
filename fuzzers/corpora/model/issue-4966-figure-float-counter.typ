
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#let c = context counter(figure.where(kind: image)).display()
#set align(center)

#c

#figure(
  square(c),
  placement: bottom,
  caption: [A]
)

#c

#figure(
  circle(c),
  placement: top,
  caption: [B]
)

#c
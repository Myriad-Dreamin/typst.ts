
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set text(hyphenate: false, overhang: false)
#set par(
  justify: true,
  justification-limits: (
    spacing: (min: 100%, max: 100%),
    tracking: (min: 0em, max: 0.5em)
  )
)

#block(fill: aqua.lighten(50%), width: 100%, lorem(10))
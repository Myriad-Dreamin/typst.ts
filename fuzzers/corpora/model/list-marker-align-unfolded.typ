
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Marker align option should not be affected by the context.
#[
  #set align(top)
  #set list(marker-align: horizon)

  - #box(fill: teal, inset: 10pt )[]
]

#[
  #set align(horizon)
  - #box(fill: teal, inset: 10pt)[]
]
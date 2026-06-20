
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Multiple fonts with multiple different axis combinations in one test.
#text(font: "Roboto Flex")[
  Roboto _Flex_
  #text(variations: (GRAD: 150))[
    with #text(stretch: 150%)[*Grade* axis] enabled
  ]
] \
#text(font: "Source Serif 4")[
  Source _Serif_ 4 *Variable*
]
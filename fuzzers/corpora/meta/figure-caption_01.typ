
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test figure.caption element for specific figure kinds
#show figure.caption.where(kind: table): underline

#figure(
  [Not a table],
  caption: [Not underlined],
)

#figure(
  table[A table],
  caption: [Underlined],
)

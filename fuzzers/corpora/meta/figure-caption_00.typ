
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test figure.caption element
#show figure.caption: emph

#figure(
  [Not italicized],
  caption: [Italicized],
)

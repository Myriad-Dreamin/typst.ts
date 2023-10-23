
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test custom separator for figure caption
#set figure.caption(separator: [ --- ])

#figure(
  table(columns: 2)[a][b],
  caption: [The table with custom separator.],
)

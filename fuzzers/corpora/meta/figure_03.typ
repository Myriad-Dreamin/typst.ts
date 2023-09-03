
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test breakable figures
#set page(height: 6em)
#show figure: set block(breakable: true)

#figure(table[a][b][c][d][e], caption: [A table])

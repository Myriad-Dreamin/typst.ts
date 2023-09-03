
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test joining.
#for word in lorem(4).split().map(s => s.trim(".")) [
  / #word: Latin stuff.
]

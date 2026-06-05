
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test numbering of empty regions of broken equations.
#set page(height: 5em)
#set math.equation(numbering: "1")
#show math.equation: set block(breakable: true)

#rect(height: 1.5em)

$ a + b \
  a + b $
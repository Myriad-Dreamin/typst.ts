
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test breaking of single line equations with numbering.
#set page(height: 4em)
#show math.equation: set block(breakable: true)
#set math.equation(numbering: "(1)")

Shouldn't overflow:
$ a + b $
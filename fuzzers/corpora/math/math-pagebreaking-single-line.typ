
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test breaking of single line equations.
#set page(height: 4em)
#show math.equation: set block(breakable: true)

Shouldn't overflow:
$ a + b $

#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test empty breakable equations.
#show math.equation: set block(breakable: true)
#math.equation(block: true, [])
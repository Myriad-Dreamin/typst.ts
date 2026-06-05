
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test numbering on empty equations.
#math.equation(numbering: "1", block: true, [])
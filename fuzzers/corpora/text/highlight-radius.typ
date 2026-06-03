
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test highlight radius
#highlight(radius: 3pt)[abc],
#highlight(radius: 1em)[#lorem(5)]
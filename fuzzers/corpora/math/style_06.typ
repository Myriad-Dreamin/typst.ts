
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test different font.
#show math.equation: set text(font: "Fira Math")
$ v := vec(1 + 2, 2 - 4, sqrt(3), arrow(x)) + 1 $

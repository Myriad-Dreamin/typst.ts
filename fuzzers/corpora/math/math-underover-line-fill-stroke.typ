
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test that the horizontal stroke is also decorated like text glyphs
#text(size: 20pt, fill: yellow, stroke: red + .5pt)[$underline(Delta).overline(Delta)$]
#text(size: 25pt, stroke: red)[$underline(Delta).overline(Delta)$]
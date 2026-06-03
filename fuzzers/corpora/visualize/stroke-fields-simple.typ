
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test stroke fields for simple strokes.
#test((1em + blue).paint, blue)
#test((1em + blue).thickness, 1em)
#test((1em + blue).cap, auto)
#test((1em + blue).join, auto)
#test((1em + blue).dash, auto)
#test((1em + blue).miter-limit, auto)

#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test that glyph assembly doesn't end up with negative lengths if the max
// overlap calculated is less than the minConnectorOverlap.
#show math.equation: set text(font: "STIX Two Math")
// Warning: glyph has assembly parts with overlap less than minConnectorOverlap
// Hint: its rendering may appear broken - this is probably a font bug
// Hint: please file an issue at https://github.com/typst/typst/issues
$ stretch(->)^"Gauss-Jordan Elimination" $
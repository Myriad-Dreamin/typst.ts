
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test that a run consisting only of whitespace isn't trimmed.
A#text(font: "IBM Plex Serif")[ ]B

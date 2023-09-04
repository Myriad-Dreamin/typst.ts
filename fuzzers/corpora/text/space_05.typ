
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test that space at start of non-backslash-linebreak line isn't trimmed.
A#"\n" B

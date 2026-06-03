
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test that eval shows warnings from the executed code.
// Warning: 7-11 no text within stars
// Hint: 7-11 using multiple consecutive stars (e.g. **) has no additional effect
#eval("**", mode: "markup")
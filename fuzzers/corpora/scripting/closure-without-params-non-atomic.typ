
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Don't parse closure directly in content.

#let x = "x"

// Should output `x => y`.
#x => y
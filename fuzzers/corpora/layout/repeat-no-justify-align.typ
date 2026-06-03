
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test repeat with alignment and disabled justification.
#set repeat(justify: false)
#set align(right)
A#box(width: 1fr, repeat(rect(width: 2em, height: 1em), gap: 1em))B
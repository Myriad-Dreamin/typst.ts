
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test that overflow does not lead to bad bounds in paragraph optimization.
#set par(justify: true)
#block(width: 0pt)[A B]